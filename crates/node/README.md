# hpc-node

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-node.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-node.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-node.svg)](https://crates.io/crates/hpc-node)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Shared contracts for node-level resource management between HPC systems. This crate defines traits and types for cgroup v2 management, Linux namespace handoff, mount lifecycle, and boot readiness signaling.

This crate enables multiple applications (like [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice)) to share common node management conventions while implementing their own backends independently.

## Features

- **cgroup v2 Conventions**: Shared slice naming (`pact.slice/`, `workload.slice/`), ownership model, scope management trait, resource limits
- **Namespace Handoff**: Protocol for passing Linux namespace FDs between processes via unix socket (`SCM_RIGHTS`)
- **Mount Management**: Refcounted mount trait with lazy unmount and crash-recovery reconstruction
- **Readiness Signaling**: Boot readiness gate trait for coordinating init and workload systems

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hpc-node = "2026.1"
```

Or to use the latest development version from git:

```toml
[dependencies]
hpc-node = { git = "https://github.com/witlox/hpc-core" }
```

## Usage

### 1. Implement the CgroupManager Trait

Each application provides its own cgroup management backend:

```rust
use hpc_node::{CgroupManager, CgroupHandle, CgroupError, CgroupMetrics, ResourceLimits, slices};

struct MyCgroupManager;

impl CgroupManager for MyCgroupManager {
    fn create_hierarchy(&self) -> Result<(), CgroupError> {
        // Create pact.slice/ and workload.slice/ in cgroup v2 filesystem
        todo!()
    }

    fn create_scope(
        &self,
        parent_slice: &str,
        name: &str,
        limits: &ResourceLimits,
    ) -> Result<CgroupHandle, CgroupError> {
        // Create a scoped cgroup under parent_slice with resource limits
        todo!()
    }

    fn destroy_scope(&self, handle: &CgroupHandle) -> Result<(), CgroupError> {
        // Kill all processes in scope via cgroup.kill, then remove
        todo!()
    }

    fn read_metrics(&self, path: &str) -> Result<CgroupMetrics, CgroupError> {
        // Read memory.current, cpu.stat, cgroup.procs from cgroup filesystem
        todo!()
    }

    fn is_scope_empty(&self, handle: &CgroupHandle) -> Result<bool, CgroupError> {
        // Check if cgroup.procs is empty
        todo!()
    }
}
```

### 2. Query Slice Ownership

```rust
use hpc_node::{SliceOwner, cgroup::slice_owner, cgroup::slices};

// Determine who owns a cgroup path
assert_eq!(slice_owner(slices::PACT_GPU), Some(SliceOwner::Pact));
assert_eq!(slice_owner(slices::WORKLOAD_ROOT), Some(SliceOwner::Workload));
```

### 3. Namespace Handoff

```rust
use hpc_node::{NamespaceRequest, NamespaceType, namespace::HANDOFF_SOCKET_PATH};

// Lattice requests namespaces from pact via unix socket
let request = NamespaceRequest {
    allocation_id: "alloc-42".to_string(),
    namespaces: vec![NamespaceType::Pid, NamespaceType::Net, NamespaceType::Mount],
    uenv_image: Some("pytorch-2.5.sqfs".to_string()),
};
// Send request over HANDOFF_SOCKET_PATH, receive FDs via SCM_RIGHTS
```

### 4. Mount Refcounting

```rust
use hpc_node::MountManager;

// Implementer provides refcounted mount management
// acquire_mount() → increments refcount (mounts if first)
// release_mount() → decrements refcount (starts hold timer at zero)
// force_unmount() → immediate unmount (emergency mode only)
// reconstruct_state() → rebuild refcounts from /proc/mounts after crash
```

## Architecture

### What's Provided (Shared Contract)

| Component | Description |
|-----------|-------------|
| `CgroupManager` trait | Hierarchy creation, scope lifecycle, metrics reading |
| `SliceOwner` enum | Ownership model: Pact (system services) vs Workload (allocations) |
| `slices` constants | Well-known cgroup paths for consistent hierarchy |
| `NamespaceProvider` / `NamespaceConsumer` traits | FD handoff protocol |
| `MountManager` trait | Refcounted mount lifecycle with lazy unmount |
| `ReadinessGate` trait | Boot readiness signaling |
| Well-known paths | Socket paths, mount base directories |

### What You Provide (Application-Specific)

| Component | Description |
|-----------|-------------|
| `CgroupManager` impl | Your cgroup v2 filesystem operations |
| `NamespaceProvider` impl | Your `unshare(2)` + FD management |
| `MountManager` impl | Your `mount(2)` + refcount tracking |
| `ReadinessGate` impl | Your boot sequence completion signal |

### Design Principles

- **Traits and types only** — no Linux-specific code, no implementations
- **No runtime coupling** — pact and lattice have no runtime dependency on each other
- **Convention over configuration** — well-known paths prevent drift
- **Both systems work independently** — lattice creates its own hierarchy when pact is absent
