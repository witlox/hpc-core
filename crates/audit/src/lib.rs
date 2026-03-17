//! # hpc-audit
//!
//! Shared audit event types and sink trait for HPC systems.
//!
//! Provides a universal [`AuditEvent`] format and [`AuditSink`] trait
//! that both pact and lattice implement. Loose coupling, high coherence —
//! each system owns its audit log, shared format enables unified SIEM forwarding.
//!
//! ## Design
//!
//! - `AuditSink::emit()` must not block — buffer internally
//! - Each system maintains its own audit log (pact → journal, lattice → quorum)
//! - Shared `AuditEvent` format enables a single SIEM forwarder for both systems
//! - `AuditSource` enum distinguishes which system emitted an event

mod event;
mod policy;
mod sink;

pub use event::{AuditEvent, AuditOutcome, AuditPrincipal, AuditScope, AuditSource, PrincipalType};
pub use policy::CompliancePolicy;
pub use sink::{AuditError, AuditSink, MemoryAuditSink, NullAuditSink};

/// Well-known action strings for cross-system consistency.
///
/// Both pact and lattice use these constants to ensure audit events
/// are consistently categorized regardless of which system emits them.
pub mod actions {
    // Process supervision
    pub const SERVICE_START: &str = "service.start";
    pub const SERVICE_STOP: &str = "service.stop";
    pub const SERVICE_RESTART: &str = "service.restart";
    pub const SERVICE_CRASH: &str = "service.crash";

    // Resource isolation
    pub const CGROUP_CREATE: &str = "cgroup.create";
    pub const CGROUP_DESTROY: &str = "cgroup.destroy";
    pub const CGROUP_KILL_FAILED: &str = "cgroup.kill_failed";
    pub const EMERGENCY_FREEZE: &str = "emergency.freeze";
    pub const EMERGENCY_KILL: &str = "emergency.kill";

    // Identity mapping
    pub const UID_ASSIGNED: &str = "identity.uid_assigned";
    pub const UID_RANGE_EXHAUSTED: &str = "identity.range_exhausted";
    pub const FEDERATION_DEPARTURE_GC: &str = "identity.federation_gc";

    // Workload integration
    pub const NAMESPACE_HANDOFF: &str = "namespace.handoff";
    pub const NAMESPACE_HANDOFF_FAILED: &str = "namespace.handoff_failed";
    pub const NAMESPACE_CLEANUP: &str = "namespace.cleanup";
    pub const NAMESPACE_LEAK_DETECTED: &str = "namespace.leak_detected";
    pub const MOUNT_ACQUIRE: &str = "mount.acquire";
    pub const MOUNT_RELEASE: &str = "mount.release";
    pub const MOUNT_FORCE_UNMOUNT: &str = "mount.force_unmount";
    pub const MOUNT_REFCOUNT_CORRECTED: &str = "mount.refcount_corrected";

    // Network
    pub const NETWORK_CONFIGURED: &str = "network.configured";
    pub const NETWORK_FAILED: &str = "network.failed";
    pub const NETWORK_LINK_LOST: &str = "network.link_lost";

    // Bootstrap
    pub const BOOT_PHASE_COMPLETE: &str = "boot.phase_complete";
    pub const BOOT_PHASE_FAILED: &str = "boot.phase_failed";
    pub const BOOT_READY: &str = "boot.ready";
    pub const WATCHDOG_TIMEOUT: &str = "boot.watchdog_timeout";
    pub const SPIRE_SVID_ACQUIRED: &str = "boot.spire_svid_acquired";

    // Configuration management
    pub const CONFIG_COMMIT: &str = "config.commit";
    pub const CONFIG_ROLLBACK: &str = "config.rollback";
    pub const CONFIG_DRIFT_DETECTED: &str = "config.drift_detected";

    // Admin operations
    pub const EXEC_COMMAND: &str = "admin.exec";
    pub const SHELL_SESSION_START: &str = "admin.shell_start";
    pub const SHELL_SESSION_END: &str = "admin.shell_end";
    pub const EMERGENCY_START: &str = "admin.emergency_start";
    pub const EMERGENCY_END: &str = "admin.emergency_end";

    // Enrollment
    pub const NODE_ENROLLED: &str = "enrollment.enrolled";
    pub const NODE_ACTIVATED: &str = "enrollment.activated";
    pub const NODE_DECOMMISSIONED: &str = "enrollment.decommissioned";
    pub const CERT_RENEWED: &str = "enrollment.cert_renewed";

    // Workload lifecycle (lattice-specific)
    pub const ALLOCATION_START: &str = "workload.allocation_start";
    pub const ALLOCATION_END: &str = "workload.allocation_end";
    pub const CHECKPOINT_TRIGGERED: &str = "workload.checkpoint_triggered";
    pub const PREEMPTION: &str = "workload.preemption";
}
