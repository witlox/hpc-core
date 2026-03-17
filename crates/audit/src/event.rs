//! Audit event types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Universal audit event. Both pact and lattice emit these.
///
/// Every field is required. Use empty strings for unknown values rather
/// than `Option` — this ensures SIEM systems always receive consistent
/// event shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID (UUID v4 recommended).
    pub id: String,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Who performed the action.
    pub principal: AuditPrincipal,
    /// What action was performed (use `actions::*` constants).
    pub action: String,
    /// Where (node, vCluster, allocation).
    pub scope: AuditScope,
    /// Success, failure, or denied.
    pub outcome: AuditOutcome,
    /// Human-readable detail message.
    pub detail: String,
    /// Structured metadata (action-specific). Use `serde_json::Value::Null` if none.
    pub metadata: serde_json::Value,
    /// Which system emitted this event.
    pub source: AuditSource,
}

/// Who performed the action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPrincipal {
    /// OIDC subject, service account, or system identifier.
    pub identity: String,
    /// Principal type.
    pub principal_type: PrincipalType,
    /// Role at time of action (e.g., `pact-ops-ml-training`).
    pub role: String,
}

impl AuditPrincipal {
    /// Create a system principal (for internal pact/lattice operations).
    #[must_use]
    pub fn system(identity: &str) -> Self {
        Self {
            identity: identity.to_string(),
            principal_type: PrincipalType::System,
            role: String::new(),
        }
    }
}

/// Principal type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrincipalType {
    /// Human administrator.
    Human,
    /// AI agent (e.g., Claude via MCP).
    Agent,
    /// Machine service account (e.g., pact-service-agent).
    Service,
    /// Internal system operation (e.g., supervision loop restart).
    System,
}

/// Where the action occurred.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct AuditScope {
    /// Node identifier (if node-scoped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// vCluster identifier (if vCluster-scoped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcluster_id: Option<String>,
    /// Allocation identifier (if allocation-scoped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_id: Option<String>,
}

impl AuditScope {
    /// Node-scoped event.
    #[must_use]
    pub fn node(node_id: &str) -> Self {
        Self {
            node_id: Some(node_id.to_string()),
            ..Default::default()
        }
    }

    /// Node + vCluster scoped event.
    #[must_use]
    pub fn node_vcluster(node_id: &str, vcluster_id: &str) -> Self {
        Self {
            node_id: Some(node_id.to_string()),
            vcluster_id: Some(vcluster_id.to_string()),
            ..Default::default()
        }
    }
}

/// Event outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditOutcome {
    /// Action completed successfully.
    Success,
    /// Action failed (error, not authorization).
    Failure,
    /// Action denied by policy.
    Denied,
}

/// Which system emitted the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditSource {
    PactAgent,
    PactJournal,
    PactCli,
    LatticeNodeAgent,
    LatticeQuorum,
    LatticeCli,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_serialization_roundtrip() {
        let event = AuditEvent {
            id: "test-123".to_string(),
            timestamp: Utc::now(),
            principal: AuditPrincipal {
                identity: "admin@example.com".to_string(),
                principal_type: PrincipalType::Human,
                role: "pact-ops-ml-training".to_string(),
            },
            action: crate::actions::SERVICE_RESTART.to_string(),
            scope: AuditScope::node("node-001"),
            outcome: AuditOutcome::Success,
            detail: "Restarted nvidia-persistenced after crash".to_string(),
            metadata: serde_json::json!({"service": "nvidia-persistenced", "restart_count": 3}),
            source: AuditSource::PactAgent,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deser: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, "test-123");
        assert_eq!(deser.action, "service.restart");
        assert_eq!(deser.principal.identity, "admin@example.com");
        assert_eq!(deser.scope.node_id.as_deref(), Some("node-001"));
        assert_eq!(deser.outcome, AuditOutcome::Success);
        assert_eq!(deser.source, AuditSource::PactAgent);
    }

    #[test]
    fn system_principal() {
        let p = AuditPrincipal::system("supervision-loop");
        assert_eq!(p.principal_type, PrincipalType::System);
        assert_eq!(p.identity, "supervision-loop");
        assert!(p.role.is_empty());
    }

    #[test]
    fn scope_constructors() {
        let s = AuditScope::node("n1");
        assert_eq!(s.node_id.as_deref(), Some("n1"));
        assert!(s.vcluster_id.is_none());

        let s = AuditScope::node_vcluster("n1", "ml-training");
        assert_eq!(s.node_id.as_deref(), Some("n1"));
        assert_eq!(s.vcluster_id.as_deref(), Some("ml-training"));
    }

    #[test]
    fn scope_none_fields_skip_in_json() {
        let s = AuditScope::node("n1");
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("vcluster_id"));
        assert!(!json.contains("allocation_id"));
    }

    #[test]
    fn audit_event_with_null_metadata() {
        let event = AuditEvent {
            id: "test".to_string(),
            timestamp: Utc::now(),
            principal: AuditPrincipal::system("test"),
            action: crate::actions::BOOT_READY.to_string(),
            scope: AuditScope::default(),
            outcome: AuditOutcome::Success,
            detail: String::new(),
            metadata: serde_json::Value::Null,
            source: AuditSource::PactAgent,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deser: AuditEvent = serde_json::from_str(&json).unwrap();
        assert!(deser.metadata.is_null());
    }

    #[test]
    fn all_outcomes_serialize() {
        for outcome in [AuditOutcome::Success, AuditOutcome::Failure, AuditOutcome::Denied] {
            let json = serde_json::to_string(&outcome).unwrap();
            let deser: AuditOutcome = serde_json::from_str(&json).unwrap();
            assert_eq!(deser, outcome);
        }
    }

    #[test]
    fn all_sources_serialize() {
        for source in [
            AuditSource::PactAgent,
            AuditSource::PactJournal,
            AuditSource::PactCli,
            AuditSource::LatticeNodeAgent,
            AuditSource::LatticeQuorum,
            AuditSource::LatticeCli,
        ] {
            let json = serde_json::to_string(&source).unwrap();
            let deser: AuditSource = serde_json::from_str(&json).unwrap();
            assert_eq!(deser, source);
        }
    }

    #[test]
    fn all_principal_types_serialize() {
        for pt in [
            PrincipalType::Human,
            PrincipalType::Agent,
            PrincipalType::Service,
            PrincipalType::System,
        ] {
            let json = serde_json::to_string(&pt).unwrap();
            let deser: PrincipalType = serde_json::from_str(&json).unwrap();
            assert_eq!(deser, pt);
        }
    }

    #[test]
    fn scope_default_is_empty() {
        let s = AuditScope::default();
        assert!(s.node_id.is_none());
        assert!(s.vcluster_id.is_none());
        assert!(s.allocation_id.is_none());
    }
}
