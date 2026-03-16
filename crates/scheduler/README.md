# hpc-scheduler-core

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-scheduler.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-scheduler.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-scheduler-core.svg)](https://crates.io/crates/hpc-scheduler-core)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Core scheduling algorithms for HPC job schedulers. This crate provides generic, reusable components for building topology-aware, multi-factor schedulers with reservation-based backfill.

This crate enables multiple applications (like [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice)) to share common scheduling infrastructure while defining their own job and node types.

## Features

- **Knapsack Solver**: Greedy multi-dimensional placement with reservation-based backfill
- **Composite Cost Function**: 9-factor scoring (priority, wait time, fair share, topology, data readiness, backlog, energy, checkpoint efficiency, conformance)
- **Topology-Aware Placement**: Dragonfly group packing with tight/spread/any preferences
- **Conformance Grouping**: Hardware homogeneity scoring and constraint filtering
- **Preemption**: Class-based victim selection with sensitive job protection
- **Walltime Enforcement**: Two-phase termination (SIGTERM then SIGKILL)
- **Resource Timeline**: Future node-release tracking for reservation and backfill scheduling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hpc-scheduler-core = "2026.1"
```

With serde support:

```toml
[dependencies]
hpc-scheduler-core = { version = "2026.1", features = ["serde"] }
```

Or to use the latest development version from git:

```toml
[dependencies]
hpc-scheduler-core = { git = "https://github.com/witlox/hpc-core" }
```

## Usage

### 1. Implement the Job and ComputeNode Traits

Each application provides its own job and node types:

```rust
use hpc_scheduler_core::{Job, ComputeNode};
use chrono::{DateTime, Utc};
use uuid::Uuid;

struct MyJob {
    id: Uuid,
    tenant: String,
    nodes_needed: u32,
    // ...
}

impl Job for MyJob {
    fn id(&self) -> Uuid { self.id }
    fn tenant_id(&self) -> &str { &self.tenant }
    fn node_count_min(&self) -> u32 { self.nodes_needed }
    // ... implement remaining methods
}
```

### 2. Run a Scheduling Cycle

```rust
use hpc_scheduler_core::*;

let solver = KnapsackSolver::new(CostWeights::default());
let result = solver.solve(&pending_jobs, &nodes, &topology, &ctx, &timeline);

for decision in &result.decisions {
    match decision {
        PlacementDecision::Place { allocation_id, nodes } => { /* assign */ }
        PlacementDecision::Backfill { allocation_id, nodes } => { /* backfill */ }
        PlacementDecision::Preempt { allocation_id, nodes, victims } => { /* preempt */ }
        PlacementDecision::Defer { allocation_id, reason } => { /* requeue */ }
    }
}
```

### 3. Walltime Enforcement

```rust
use hpc_scheduler_core::{WalltimeEnforcer, ExpiryPhase};

let mut enforcer = WalltimeEnforcer::new();
enforcer.register(job_id, started_at, walltime, grace_period);

for expiry in enforcer.check_expired(now) {
    match expiry.phase {
        ExpiryPhase::Terminate => { /* send SIGTERM */ }
        ExpiryPhase::Kill => { /* send SIGKILL */ }
    }
}
```

## Architecture

### What's Provided (Generic)

| Component | Description |
|-----------|-------------|
| `KnapsackSolver` | 2-pass greedy placement with reservation-based backfill |
| `CostEvaluator` | 9-factor composite scoring with tunable weights |
| `WalltimeEnforcer` | Job walltime tracking with 2-phase termination |
| `ResourceTimeline` | Future node-release tracking for backfill safety checks |
| `evaluate_preemption` | Class-based victim selection |
| `select_nodes_topology_aware` | Dragonfly-aware node selection |
| `group_by_conformance` / `filter_by_constraints` | Hardware homogeneity enforcement |

### What You Provide (Application-Specific)

| Component | Description |
|-----------|-------------|
| `Job` impl | Your workload type (job, allocation, pod, etc.) |
| `ComputeNode` impl | Your node type with topology and hardware info |
| `CostContext` | Per-cycle context (tenant usage, budgets, energy price, etc.) |
| `TopologyModel` | Cluster topology (groups and adjacency) |
