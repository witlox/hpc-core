# hpc-core

[![codecov](https://codecov.io/gh/witlox/hpc-core/graph/badge.svg)](https://codecov.io/gh/witlox/hpc-core)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Shared infrastructure crates for HPC systems, used by [Lattice](https://github.com/witlox/lattice) and [Pact](https://github.com/witlox/pact).

## Crates

| Crate | Description | crates.io |
|-------|-------------|-----------|
| [raft-hpc-core](crates/raft/) | Raft consensus infrastructure built on [openraft](https://github.com/datafuselabs/openraft) — log stores, gRPC transport, state machine, backup/restore | [![crates.io](https://img.shields.io/crates/v/raft-hpc-core.svg)](https://crates.io/crates/raft-hpc-core) |
| [hpc-scheduler-core](crates/scheduler/) | Scheduling algorithms — knapsack solver, topology-aware placement, backfill, preemption, walltime enforcement | [![crates.io](https://img.shields.io/crates/v/hpc-scheduler-core.svg)](https://crates.io/crates/hpc-scheduler-core) |
| [hpc-auth](crates/auth/) | OAuth2/OIDC authentication — multi-flow support, per-server token caching, automatic refresh, PKCE | [![crates.io](https://img.shields.io/crates/v/hpc-auth.svg)](https://crates.io/crates/hpc-auth) |

## Development

```bash
cargo install just

just check     # type-check workspace
just fmt       # format all code
just lint      # clippy
just test      # run tests
just deny      # license & advisory checks
just all       # full local CI
```
