//! # hpc-node
//!
//! Shared contracts for node-level resource management between HPC systems.
//!
//! This crate defines traits and types for:
//! - **cgroup v2 conventions** (`cgroup`): slice naming, ownership, scope management
//! - **Namespace handoff** (`namespace`): protocol for passing namespace FDs between processes
//! - **Mount management** (`mount`): refcounted mounts with lazy unmount
//! - **Readiness signaling** (`readiness`): boot readiness gate
//!
//! Both pact-agent and lattice-node-agent implement these traits independently.
//! When pact is init, lattice gains capabilities ("steroids") through the handoff
//! protocol. When lattice runs standalone, it creates its own hierarchy using the
//! same conventions.
//!
//! ## Design principles
//!
//! - **Traits and types only** — no implementations, no Linux-specific code
//! - **No runtime coupling** — pact and lattice have no runtime dependency on each other
//! - **Convention over configuration** — well-known paths and constants prevent drift

pub mod cgroup;
pub mod mount;
pub mod namespace;
pub mod readiness;

// Re-export key types at crate root.
pub use cgroup::{
    CgroupError, CgroupHandle, CgroupManager, CgroupMetrics, ResourceLimits, SliceOwner,
};
pub use mount::{MountError, MountHandle, MountManager};
pub use namespace::{
    NamespaceConsumer, NamespaceError, NamespaceProvider, NamespaceRequest, NamespaceResponse,
    NamespaceType,
};
pub use readiness::{ReadinessError, ReadinessGate};
