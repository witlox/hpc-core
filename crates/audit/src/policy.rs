//! Compliance policy types.

use serde::{Deserialize, Serialize};

use crate::actions;

/// Retention and compliance requirements for audit events.
///
/// Configurable per vCluster. Regulated/sensitive vClusters use
/// `regulated()` with 7-year retention and full access logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    /// Minimum retention period for audit events in days.
    pub retention_days: u32,
    /// Whether all access operations must be logged (not just mutations).
    pub log_all_access: bool,
    /// Actions that MUST emit audit events. Missing events are a compliance violation.
    pub required_audit_points: Vec<String>,
}

impl Default for CompliancePolicy {
    fn default() -> Self {
        Self {
            retention_days: 365,
            log_all_access: false,
            required_audit_points: vec![
                actions::SERVICE_CRASH.to_string(),
                actions::EMERGENCY_FREEZE.to_string(),
                actions::EMERGENCY_KILL.to_string(),
                actions::UID_ASSIGNED.to_string(),
                actions::NAMESPACE_HANDOFF_FAILED.to_string(),
                actions::BOOT_PHASE_FAILED.to_string(),
                actions::NODE_ENROLLED.to_string(),
                actions::NODE_DECOMMISSIONED.to_string(),
                actions::CONFIG_COMMIT.to_string(),
                actions::CONFIG_ROLLBACK.to_string(),
            ],
        }
    }
}

impl CompliancePolicy {
    /// Policy for regulated/sensitive vClusters.
    ///
    /// 7-year retention, full access logging, all actions audited.
    #[must_use]
    pub fn regulated() -> Self {
        Self {
            retention_days: 2555, // ~7 years
            log_all_access: true,
            required_audit_points: vec![
                // All default points
                actions::SERVICE_START.to_string(),
                actions::SERVICE_STOP.to_string(),
                actions::SERVICE_RESTART.to_string(),
                actions::SERVICE_CRASH.to_string(),
                actions::CGROUP_CREATE.to_string(),
                actions::CGROUP_DESTROY.to_string(),
                actions::CGROUP_KILL_FAILED.to_string(),
                actions::EMERGENCY_FREEZE.to_string(),
                actions::EMERGENCY_KILL.to_string(),
                actions::UID_ASSIGNED.to_string(),
                actions::UID_RANGE_EXHAUSTED.to_string(),
                actions::NAMESPACE_HANDOFF.to_string(),
                actions::NAMESPACE_HANDOFF_FAILED.to_string(),
                actions::NAMESPACE_CLEANUP.to_string(),
                actions::MOUNT_ACQUIRE.to_string(),
                actions::MOUNT_RELEASE.to_string(),
                actions::MOUNT_FORCE_UNMOUNT.to_string(),
                actions::NETWORK_CONFIGURED.to_string(),
                actions::NETWORK_FAILED.to_string(),
                actions::BOOT_PHASE_COMPLETE.to_string(),
                actions::BOOT_PHASE_FAILED.to_string(),
                actions::BOOT_READY.to_string(),
                actions::CONFIG_COMMIT.to_string(),
                actions::CONFIG_ROLLBACK.to_string(),
                actions::CONFIG_DRIFT_DETECTED.to_string(),
                actions::EXEC_COMMAND.to_string(),
                actions::SHELL_SESSION_START.to_string(),
                actions::SHELL_SESSION_END.to_string(),
                actions::EMERGENCY_START.to_string(),
                actions::EMERGENCY_END.to_string(),
                actions::NODE_ENROLLED.to_string(),
                actions::NODE_DECOMMISSIONED.to_string(),
                actions::CERT_RENEWED.to_string(),
            ],
        }
    }

    /// Check if a given action is a required audit point.
    #[must_use]
    pub fn is_required(&self, action: &str) -> bool {
        self.required_audit_points.iter().any(|a| a == action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy() {
        let policy = CompliancePolicy::default();
        assert_eq!(policy.retention_days, 365);
        assert!(!policy.log_all_access);
        assert!(policy.is_required(actions::SERVICE_CRASH));
        assert!(!policy.is_required(actions::SERVICE_START));
    }

    #[test]
    fn regulated_policy() {
        let policy = CompliancePolicy::regulated();
        assert_eq!(policy.retention_days, 2555);
        assert!(policy.log_all_access);
        assert!(policy.is_required(actions::SERVICE_START));
        assert!(policy.is_required(actions::EXEC_COMMAND));
        assert!(policy.is_required(actions::CERT_RENEWED));
    }

    #[test]
    fn is_required_unknown_action() {
        let policy = CompliancePolicy::default();
        assert!(!policy.is_required("totally.unknown.action"));
    }

    #[test]
    fn regulated_is_superset_of_default() {
        let default = CompliancePolicy::default();
        let regulated = CompliancePolicy::regulated();
        for action in &default.required_audit_points {
            assert!(
                regulated.is_required(action),
                "regulated policy missing default required action: {action}"
            );
        }
    }

    #[test]
    fn policy_serialization_roundtrip() {
        let policy = CompliancePolicy::regulated();
        let json = serde_json::to_string(&policy).unwrap();
        let deser: CompliancePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.retention_days, 2555);
        assert!(deser.log_all_access);
        assert_eq!(
            deser.required_audit_points.len(),
            policy.required_audit_points.len()
        );
    }

    #[test]
    fn action_constants_are_unique() {
        let all_actions = vec![
            actions::SERVICE_START,
            actions::SERVICE_STOP,
            actions::SERVICE_RESTART,
            actions::SERVICE_CRASH,
            actions::CGROUP_CREATE,
            actions::CGROUP_DESTROY,
            actions::CGROUP_KILL_FAILED,
            actions::EMERGENCY_FREEZE,
            actions::EMERGENCY_KILL,
            actions::UID_ASSIGNED,
            actions::UID_RANGE_EXHAUSTED,
            actions::FEDERATION_DEPARTURE_GC,
            actions::NAMESPACE_HANDOFF,
            actions::NAMESPACE_HANDOFF_FAILED,
            actions::NAMESPACE_CLEANUP,
            actions::NAMESPACE_LEAK_DETECTED,
            actions::MOUNT_ACQUIRE,
            actions::MOUNT_RELEASE,
            actions::MOUNT_FORCE_UNMOUNT,
            actions::MOUNT_REFCOUNT_CORRECTED,
            actions::NETWORK_CONFIGURED,
            actions::NETWORK_FAILED,
            actions::NETWORK_LINK_LOST,
            actions::BOOT_PHASE_COMPLETE,
            actions::BOOT_PHASE_FAILED,
            actions::BOOT_READY,
            actions::WATCHDOG_TIMEOUT,
            actions::SPIRE_SVID_ACQUIRED,
            actions::CONFIG_COMMIT,
            actions::CONFIG_ROLLBACK,
            actions::CONFIG_DRIFT_DETECTED,
            actions::EXEC_COMMAND,
            actions::SHELL_SESSION_START,
            actions::SHELL_SESSION_END,
            actions::EMERGENCY_START,
            actions::EMERGENCY_END,
            actions::NODE_ENROLLED,
            actions::NODE_ACTIVATED,
            actions::NODE_DECOMMISSIONED,
            actions::CERT_RENEWED,
            actions::ALLOCATION_START,
            actions::ALLOCATION_END,
            actions::CHECKPOINT_TRIGGERED,
            actions::PREEMPTION,
        ];
        let mut seen = std::collections::HashSet::new();
        for action in &all_actions {
            assert!(
                seen.insert(action),
                "duplicate action constant: {action}"
            );
        }
    }
}
