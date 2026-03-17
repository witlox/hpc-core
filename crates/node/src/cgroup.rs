//! cgroup v2 conventions and management trait.
//!
//! Defines the shared cgroup hierarchy layout that both pact and lattice use,
//! regardless of which system creates it.

use serde::{Deserialize, Serialize};

/// cgroup slice ownership — who has exclusive write access.
///
/// Invariant RI1: each slice subtree is owned by exactly one system.
/// No system writes to another's slice except during declared emergency (RI3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SliceOwner {
    /// pact-agent owns this slice (system services).
    Pact,
    /// lattice-node-agent owns this slice (workloads).
    Workload,
}

/// Well-known cgroup slice paths.
///
/// Both pact and lattice use these constants to ensure consistent hierarchy
/// regardless of which system creates the slices.
pub mod slices {
    /// Root slice for pact-managed system services.
    pub const PACT_ROOT: &str = "pact.slice";
    /// Infrastructure services: chronyd, dbus-daemon, rasdaemon.
    pub const PACT_INFRA: &str = "pact.slice/infra.slice";
    /// Network services: `cxi_rh` instances.
    pub const PACT_NETWORK: &str = "pact.slice/network.slice";
    /// GPU services: nvidia-persistenced, nv-hostengine.
    pub const PACT_GPU: &str = "pact.slice/gpu.slice";
    /// Audit services: auditd, audit-forwarder (regulated vClusters only).
    pub const PACT_AUDIT: &str = "pact.slice/audit.slice";
    /// Root slice for lattice-managed workload allocations.
    pub const WORKLOAD_ROOT: &str = "workload.slice";
}

/// Returns the owner of a given cgroup path.
///
/// Returns `None` for paths outside the known hierarchy (e.g., root cgroup).
#[must_use]
pub fn slice_owner(path: &str) -> Option<SliceOwner> {
    if path.starts_with(slices::PACT_ROOT) {
        Some(SliceOwner::Pact)
    } else if path.starts_with(slices::WORKLOAD_ROOT) {
        Some(SliceOwner::Workload)
    } else {
        None
    }
}

/// Resource limits for a cgroup scope.
///
/// Applied when creating a scope for a service or allocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in bytes (maps to `memory.max`). `None` = unlimited.
    pub memory_max: Option<u64>,
    /// CPU weight (1–10000, maps to `cpu.weight`). `None` = default (100).
    pub cpu_weight: Option<u16>,
    /// IO max in bytes/sec. `None` = unlimited.
    pub io_max: Option<u64>,
}

/// Opaque handle to a created cgroup scope.
///
/// Returned by [`CgroupManager::create_scope`] and passed to process spawn
/// for placement. Implementers store whatever is needed to reference the scope
/// (typically the cgroup path).
#[derive(Debug, Clone)]
pub struct CgroupHandle {
    /// Full cgroup path (e.g., `/sys/fs/cgroup/pact.slice/gpu.slice/nvidia-persistenced`).
    pub path: String,
}

/// Metrics read from a cgroup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CgroupMetrics {
    /// Current memory usage in bytes (`memory.current`).
    pub memory_current: u64,
    /// Memory limit in bytes (`memory.max`). `None` if unlimited.
    pub memory_max: Option<u64>,
    /// Total CPU usage in microseconds (`cpu.stat` → `usage_usec`).
    pub cpu_usage_usec: u64,
    /// Number of processes in the cgroup (`cgroup.procs` line count).
    pub nr_processes: u32,
}

/// Trait for cgroup hierarchy management.
///
/// Both pact (direct cgroup v2 filesystem) and lattice (standalone mode)
/// implement this. The trait defines the contract; ownership enforcement
/// (RI1) and emergency override (RI3) are the implementer's responsibility.
///
/// # Invariants enforced
///
/// - RI2: every supervised process has a scope (caller must use `create_scope` before spawn)
/// - RI5: callback on failure (caller must call `destroy_scope` on spawn failure)
/// - RI6: shared read (any path readable via `read_metrics`)
pub trait CgroupManager: Send + Sync {
    /// Create the top-level slice hierarchy.
    ///
    /// Called once at boot. Idempotent — safe to call if hierarchy already exists.
    /// Creates `pact.slice/` and `workload.slice/` with their sub-slices.
    fn create_hierarchy(&self) -> Result<(), CgroupError>;

    /// Create a scoped cgroup for a service or allocation.
    ///
    /// Returns a handle for process placement. The scope is created under
    /// `parent_slice` with the given `name` and resource limits applied.
    ///
    /// # Errors
    ///
    /// Returns [`CgroupError::CreationFailed`] if the scope cannot be created.
    /// Returns [`CgroupError::PermissionDenied`] if the caller doesn't own the parent slice.
    fn create_scope(
        &self,
        parent_slice: &str,
        name: &str,
        limits: &ResourceLimits,
    ) -> Result<CgroupHandle, CgroupError>;

    /// Kill all processes in a scope and release it.
    ///
    /// Uses `cgroup.kill` (Linux 5.14+) for immediate cleanup. No grace period
    /// for child processes (PS3). Falls back to iterating `cgroup.procs` + SIGKILL
    /// on older kernels.
    ///
    /// # Errors
    ///
    /// Returns [`CgroupError::KillFailed`] if processes cannot be killed (e.g., D-state).
    /// The scope should be marked as zombie in this case (F30).
    fn destroy_scope(&self, handle: &CgroupHandle) -> Result<(), CgroupError>;

    /// Read metrics from any cgroup path.
    ///
    /// Shared read access across all slices (RI6) — no ownership check.
    fn read_metrics(&self, path: &str) -> Result<CgroupMetrics, CgroupError>;

    /// Check if a scope is empty (no processes).
    ///
    /// Used by the supervision loop to detect completed allocations (WI5).
    fn is_scope_empty(&self, handle: &CgroupHandle) -> Result<bool, CgroupError>;
}

/// Errors from cgroup operations.
#[derive(Debug, thiserror::Error)]
pub enum CgroupError {
    #[error("cgroup creation failed: {reason}")]
    CreationFailed { reason: String },

    #[error("cgroup.kill failed for {path}: {reason}")]
    KillFailed { path: String, reason: String },

    #[error("cgroup path not found: {path}")]
    NotFound { path: String },

    #[error("permission denied: {path} owned by {owner:?}")]
    PermissionDenied { path: String, owner: SliceOwner },

    #[error("cgroup I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_owner_pact() {
        assert_eq!(slice_owner(slices::PACT_ROOT), Some(SliceOwner::Pact));
        assert_eq!(slice_owner(slices::PACT_INFRA), Some(SliceOwner::Pact));
        assert_eq!(slice_owner(slices::PACT_GPU), Some(SliceOwner::Pact));
        assert_eq!(slice_owner(slices::PACT_NETWORK), Some(SliceOwner::Pact));
        assert_eq!(slice_owner(slices::PACT_AUDIT), Some(SliceOwner::Pact));
    }

    #[test]
    fn slice_owner_workload() {
        assert_eq!(
            slice_owner(slices::WORKLOAD_ROOT),
            Some(SliceOwner::Workload)
        );
        assert_eq!(
            slice_owner("workload.slice/alloc-42"),
            Some(SliceOwner::Workload)
        );
    }

    #[test]
    fn slice_owner_unknown() {
        assert_eq!(slice_owner("system.slice"), None);
        assert_eq!(slice_owner(""), None);
        assert_eq!(slice_owner("/sys/fs/cgroup"), None);
    }

    #[test]
    fn resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.memory_max.is_none());
        assert!(limits.cpu_weight.is_none());
        assert!(limits.io_max.is_none());
    }

    #[test]
    fn slice_owner_nested_paths() {
        // Deep nesting still resolves to root owner
        assert_eq!(
            slice_owner("pact.slice/infra.slice/chronyd.scope"),
            Some(SliceOwner::Pact)
        );
        assert_eq!(
            slice_owner("workload.slice/alloc-42/task-1.scope"),
            Some(SliceOwner::Workload)
        );
    }

    #[test]
    fn slice_owner_substring_not_matched() {
        // "not-pact.slice" should not match pact.slice prefix
        assert_eq!(slice_owner("not-pact.slice/foo"), None);
        // "workload.slice-extra" does match because starts_with
        assert_eq!(
            slice_owner("workload.slice-extra"),
            Some(SliceOwner::Workload)
        );
    }

    #[test]
    fn slice_owner_serialization() {
        let owner = SliceOwner::Pact;
        let json = serde_json::to_string(&owner).unwrap();
        let deser: SliceOwner = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, SliceOwner::Pact);

        let owner = SliceOwner::Workload;
        let json = serde_json::to_string(&owner).unwrap();
        let deser: SliceOwner = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, SliceOwner::Workload);
    }

    #[test]
    fn resource_limits_with_values() {
        let limits = ResourceLimits {
            memory_max: Some(512 * 1024 * 1024), // 512 MB
            cpu_weight: Some(200),
            io_max: Some(100_000_000),
        };
        assert_eq!(limits.memory_max, Some(536_870_912));
        assert_eq!(limits.cpu_weight, Some(200));
        assert_eq!(limits.io_max, Some(100_000_000));
    }

    #[test]
    fn resource_limits_serialization_roundtrip() {
        let limits = ResourceLimits {
            memory_max: Some(1024),
            cpu_weight: Some(500),
            io_max: None,
        };
        let json = serde_json::to_string(&limits).unwrap();
        let deser: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.memory_max, Some(1024));
        assert_eq!(deser.cpu_weight, Some(500));
        assert!(deser.io_max.is_none());
    }

    #[test]
    fn cgroup_handle_path() {
        let handle = CgroupHandle {
            path: "/sys/fs/cgroup/pact.slice/gpu.slice/nvidia-persistenced".to_string(),
        };
        assert!(handle.path.contains("pact.slice"));
    }

    #[test]
    fn cgroup_metrics_default() {
        let metrics = CgroupMetrics::default();
        assert_eq!(metrics.memory_current, 0);
        assert!(metrics.memory_max.is_none());
        assert_eq!(metrics.cpu_usage_usec, 0);
        assert_eq!(metrics.nr_processes, 0);
    }

    #[test]
    fn cgroup_error_display() {
        let err = CgroupError::CreationFailed {
            reason: "no space".to_string(),
        };
        assert_eq!(err.to_string(), "cgroup creation failed: no space");

        let err = CgroupError::KillFailed {
            path: "/sys/fs/cgroup/test".to_string(),
            reason: "D-state".to_string(),
        };
        assert!(err.to_string().contains("D-state"));

        let err = CgroupError::PermissionDenied {
            path: "workload.slice".to_string(),
            owner: SliceOwner::Workload,
        };
        assert!(err.to_string().contains("Workload"));
    }

    // Mock implementation to verify trait is implementable
    struct MockCgroupManager;

    impl CgroupManager for MockCgroupManager {
        fn create_hierarchy(&self) -> Result<(), CgroupError> {
            Ok(())
        }
        fn create_scope(
            &self,
            parent_slice: &str,
            name: &str,
            _limits: &ResourceLimits,
        ) -> Result<CgroupHandle, CgroupError> {
            Ok(CgroupHandle {
                path: format!("{parent_slice}/{name}.scope"),
            })
        }
        fn destroy_scope(&self, _handle: &CgroupHandle) -> Result<(), CgroupError> {
            Ok(())
        }
        fn read_metrics(&self, _path: &str) -> Result<CgroupMetrics, CgroupError> {
            Ok(CgroupMetrics::default())
        }
        fn is_scope_empty(&self, _handle: &CgroupHandle) -> Result<bool, CgroupError> {
            Ok(true)
        }
    }

    #[test]
    fn mock_cgroup_manager_lifecycle() {
        let mgr = MockCgroupManager;
        mgr.create_hierarchy().unwrap();

        let handle = mgr
            .create_scope(slices::PACT_GPU, "nvidia-persistenced", &ResourceLimits::default())
            .unwrap();
        assert_eq!(handle.path, "pact.slice/gpu.slice/nvidia-persistenced.scope");

        assert!(mgr.is_scope_empty(&handle).unwrap());

        let metrics = mgr.read_metrics(&handle.path).unwrap();
        assert_eq!(metrics.nr_processes, 0);

        mgr.destroy_scope(&handle).unwrap();
    }

    #[test]
    fn mock_cgroup_manager_permission_denied() {
        struct StrictMockCgroupManager;

        impl CgroupManager for StrictMockCgroupManager {
            fn create_hierarchy(&self) -> Result<(), CgroupError> {
                Ok(())
            }
            fn create_scope(
                &self,
                parent_slice: &str,
                _name: &str,
                _limits: &ResourceLimits,
            ) -> Result<CgroupHandle, CgroupError> {
                if let Some(owner) = slice_owner(parent_slice) {
                    if owner != SliceOwner::Pact {
                        return Err(CgroupError::PermissionDenied {
                            path: parent_slice.to_string(),
                            owner,
                        });
                    }
                }
                Ok(CgroupHandle {
                    path: format!("{parent_slice}/test.scope"),
                })
            }
            fn destroy_scope(&self, _handle: &CgroupHandle) -> Result<(), CgroupError> {
                Ok(())
            }
            fn read_metrics(&self, _path: &str) -> Result<CgroupMetrics, CgroupError> {
                Ok(CgroupMetrics::default())
            }
            fn is_scope_empty(&self, _handle: &CgroupHandle) -> Result<bool, CgroupError> {
                Ok(true)
            }
        }

        let mgr = StrictMockCgroupManager;

        // Pact-owned slice: allowed
        assert!(mgr
            .create_scope(slices::PACT_INFRA, "test", &ResourceLimits::default())
            .is_ok());

        // Workload-owned slice: denied (RI1)
        let err = mgr
            .create_scope(slices::WORKLOAD_ROOT, "test", &ResourceLimits::default())
            .unwrap_err();
        assert!(matches!(err, CgroupError::PermissionDenied { .. }));
    }
}
