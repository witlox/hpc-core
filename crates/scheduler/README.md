# hpc-scheduler-core

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-scheduler.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-scheduler.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-scheduler-core.svg)](https://crates.io/crates/hpc-scheduler-core)
[![codecov](https://codecov.io/gh/witlox/hpc-core/graph/badge.svg?flag=scheduler)](https://codecov.io/gh/witlox/hpc-core)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Core scheduling algorithms for HPC job schedulers. This crate provides reusable scheduling components used by [Lattice](https://github.com/witlox/lattice) and [Pact](https://github.com/witlox/pact).

## Features

- **Knapsack solver** — greedy multi-dimensional placement with reservation-based backfill
- **Composite cost function** — 9-factor scoring (priority, wait time, fair share, topology, data readiness, backlog, energy, checkpoint efficiency, conformance)
- **Topology-aware placement** — dragonfly group packing with tight/spread/any preferences
- **Conformance grouping** — hardware homogeneity scoring and constraint filtering
- **Preemption** — class-based victim selection with sensitive job protection
- **Walltime enforcement** — 2-phase termination (SIGTERM then SIGKILL)

## Installation

```toml
[dependencies]
hpc-scheduler-core = "2026.1"
```

With serde support:

```toml
[dependencies]
hpc-scheduler-core = { version = "2026.1", features = ["serde"] }
```
