//! Audit sink trait.

use crate::AuditEvent;

/// Destination for audit events.
///
/// Implementations: journal append, file write, SIEM forward, Loki push.
///
/// # Contract
///
/// - `emit()` must not block the caller. Implementations should buffer
///   events and flush asynchronously.
/// - Audit trail continuity (O3) is the responsibility of the sink
///   implementation, not the caller.
/// - If the sink is unavailable, events must be buffered locally.
///   Dropping events silently violates O3.
pub trait AuditSink: Send + Sync {
    /// Emit an audit event.
    ///
    /// Must not block the caller. Buffer internally if the destination
    /// is slow or unavailable.
    fn emit(&self, event: AuditEvent);

    /// Flush any buffered events.
    ///
    /// Called on graceful shutdown to ensure no events are lost.
    /// May block until flush completes.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError::FlushFailed`] if buffered events cannot
    /// be delivered to the destination.
    fn flush(&self) -> Result<(), AuditError>;
}

/// Errors from audit operations.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("audit sink unavailable: {reason}")]
    SinkUnavailable { reason: String },

    #[error("audit flush failed: {reason}")]
    FlushFailed { reason: String },
}

/// A no-op audit sink that discards all events.
///
/// Useful for testing or when audit is explicitly disabled.
#[derive(Debug, Default)]
pub struct NullAuditSink;

impl AuditSink for NullAuditSink {
    fn emit(&self, _event: AuditEvent) {}

    fn flush(&self) -> Result<(), AuditError> {
        Ok(())
    }
}

/// An audit sink that collects events in memory.
///
/// Useful for testing — inspect `events()` after the test.
#[derive(Debug, Default)]
pub struct MemoryAuditSink {
    events: std::sync::Mutex<Vec<AuditEvent>>,
}

impl MemoryAuditSink {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all collected events.
    #[must_use]
    pub fn events(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .expect("audit sink lock poisoned")
            .clone()
    }

    /// Returns the number of collected events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.lock().expect("audit sink lock poisoned").len()
    }

    /// Returns true if no events have been collected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AuditSink for MemoryAuditSink {
    fn emit(&self, event: AuditEvent) {
        self.events
            .lock()
            .expect("audit sink lock poisoned")
            .push(event);
    }

    fn flush(&self) -> Result<(), AuditError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuditOutcome, AuditPrincipal, AuditScope, AuditSource};
    use chrono::Utc;

    fn test_event(action: &str) -> AuditEvent {
        AuditEvent {
            id: "test".to_string(),
            timestamp: Utc::now(),
            principal: AuditPrincipal::system("test"),
            action: action.to_string(),
            scope: AuditScope::default(),
            outcome: AuditOutcome::Success,
            detail: String::new(),
            metadata: serde_json::Value::Null,
            source: AuditSource::PactAgent,
        }
    }

    #[test]
    fn null_sink_accepts_events() {
        let sink = NullAuditSink;
        sink.emit(test_event("test.action"));
        assert!(sink.flush().is_ok());
    }

    #[test]
    fn memory_sink_collects_events() {
        let sink = MemoryAuditSink::new();
        assert!(sink.is_empty());

        sink.emit(test_event("action.one"));
        sink.emit(test_event("action.two"));

        assert_eq!(sink.len(), 2);
        let events = sink.events();
        assert_eq!(events[0].action, "action.one");
        assert_eq!(events[1].action, "action.two");
    }

    #[test]
    fn memory_sink_flush_succeeds() {
        let sink = MemoryAuditSink::new();
        sink.emit(test_event("action.one"));
        assert!(sink.flush().is_ok());
        // Flush does not clear events
        assert_eq!(sink.len(), 1);
    }

    #[test]
    fn memory_sink_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let sink = Arc::new(MemoryAuditSink::new());
        let mut handles = vec![];

        for i in 0..10 {
            let sink = Arc::clone(&sink);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    sink.emit(test_event(&format!("thread-{i}-event-{j}")));
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(sink.len(), 1000);
    }
}
