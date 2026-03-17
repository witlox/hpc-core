//! Mount management conventions and trait.
//!
//! Defines refcounted mount management for uenv `SquashFS` images.
//! Multiple allocations can share one mount. Lazy unmount with
//! configurable hold time for cache locality.

use serde::{Deserialize, Serialize};

/// Well-known mount paths.
pub mod paths {
    /// Base directory for uenv `SquashFS` mounts.
    pub const UENV_MOUNT_BASE: &str = "/run/pact/uenv";
    /// Base directory for allocation working directories.
    pub const WORKDIR_BASE: &str = "/run/pact/workdir";
    /// Base directory for data staging mounts (NFS, S3).
    pub const DATA_STAGE_BASE: &str = "/run/pact/data";
}

/// Handle to an acquired mount.
///
/// Returned by [`MountManager::acquire_mount`]. The holder must call
/// [`MountManager::release_mount`] when the allocation no longer needs
/// the mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountHandle {
    /// Path to the source image (e.g., `/images/pytorch-2.5.sqfs`).
    pub image_path: String,
    /// Where the image is mounted (e.g., `/run/pact/uenv/pytorch-2.5`).
    pub mount_point: String,
}

/// Default hold time in seconds before unmounting after refcount reaches zero.
pub const DEFAULT_HOLD_TIME_SECS: u64 = 60;

/// Trait for refcounted mount management.
///
/// Both pact (as init) and lattice (standalone mode) implement this.
///
/// # Invariants
///
/// - WI2: refcount exactly equals active allocations using the mount.
///   Refcount going negative is a bug — implementations must assert.
/// - WI3: lazy unmount with configurable hold time. Emergency `--force`
///   overrides the hold timer.
/// - WI6: on agent restart, `reconstruct_state` rebuilds refcounts
///   from the kernel mount table + active allocations.
pub trait MountManager: Send + Sync {
    /// Acquire a reference to a uenv mount.
    ///
    /// If this is the first reference, the `SquashFS` image is mounted.
    /// Otherwise, the refcount is incremented and a bind-mount is
    /// prepared for the allocation's mount namespace.
    fn acquire_mount(&self, image_path: &str) -> Result<MountHandle, MountError>;

    /// Release a reference to a mount.
    ///
    /// Decrements the refcount. When refcount reaches zero, starts
    /// the cache hold timer. The mount is not unmounted until the
    /// timer expires (or emergency force-unmount).
    fn release_mount(&self, handle: &MountHandle) -> Result<(), MountError>;

    /// Force-unmount regardless of refcount or hold timer.
    ///
    /// Only allowed during emergency mode (RI3). Cancels any running
    /// hold timer and unmounts immediately.
    fn force_unmount(&self, image_path: &str) -> Result<(), MountError>;

    /// Reconstruct refcounts from kernel mount table and active allocations.
    ///
    /// Called on agent restart (WI6). Scans `/proc/mounts` and correlates
    /// with the provided list of active allocation IDs (from journal state).
    /// Mounts without matching allocations get refcount=0 and start hold timers.
    fn reconstruct_state(&self, active_allocations: &[String]) -> Result<(), MountError>;
}

/// Errors from mount operations.
#[derive(Debug, thiserror::Error)]
pub enum MountError {
    #[error("mount failed for {image_path}: {reason}")]
    MountFailed { image_path: String, reason: String },

    #[error("unmount failed for {mount_point}: {reason}")]
    UnmountFailed {
        mount_point: String,
        reason: String,
    },

    #[error("refcount inconsistency for {image_path}: {detail}")]
    RefcountInconsistency {
        image_path: String,
        detail: String,
    },

    #[error("mount I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_handle_serialization() {
        let handle = MountHandle {
            image_path: "/images/pytorch-2.5.sqfs".to_string(),
            mount_point: "/run/pact/uenv/pytorch-2.5".to_string(),
        };
        let json = serde_json::to_string(&handle).unwrap();
        let deser: MountHandle = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.image_path, "/images/pytorch-2.5.sqfs");
        assert_eq!(deser.mount_point, "/run/pact/uenv/pytorch-2.5");
    }

    #[test]
    fn well_known_paths() {
        assert!(paths::UENV_MOUNT_BASE.starts_with("/run/pact/"));
        assert!(paths::WORKDIR_BASE.starts_with("/run/pact/"));
        assert!(paths::DATA_STAGE_BASE.starts_with("/run/pact/"));
    }
}
