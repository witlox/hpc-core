//! Boot readiness signaling.
//!
//! Defines the protocol for signaling that a node is fully initialized
//! and ready for workload allocations.

/// Well-known readiness socket path.
pub const READINESS_SOCKET_PATH: &str = "/run/pact/ready.sock";

/// Well-known readiness file path (alternative to socket).
///
/// pact-agent creates this file when all boot phases complete.
/// lattice-node-agent can poll for its existence.
pub const READINESS_FILE_PATH: &str = "/run/pact/ready";

/// Trait for readiness signaling.
///
/// pact-agent implements this as a provider (emits readiness).
/// lattice-node-agent implements this as a consumer (waits for readiness).
pub trait ReadinessGate: Send + Sync {
    /// Check if the node is ready for allocations.
    ///
    /// Returns `true` once all boot phases have completed and the
    /// readiness signal has been emitted.
    fn is_ready(&self) -> bool;

    /// Wait until the node is ready.
    ///
    /// Returns immediately if already ready. Blocks until readiness
    /// is signaled or an error occurs (e.g., boot failure).
    ///
    /// Lattice requests received before readiness should be queued,
    /// not rejected.
    fn wait_ready(&self) -> Result<(), ReadinessError>;
}

/// Errors from readiness operations.
#[derive(Debug, thiserror::Error)]
pub enum ReadinessError {
    #[error("boot failed: {reason}")]
    BootFailed { reason: String },

    #[error("timeout waiting for readiness")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_paths() {
        assert!(READINESS_SOCKET_PATH.starts_with("/run/pact/"));
        assert!(READINESS_FILE_PATH.starts_with("/run/pact/"));
    }
}
