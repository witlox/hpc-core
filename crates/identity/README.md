# hpc-identity

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-identity.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-identity.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-identity.svg)](https://crates.io/crates/hpc-identity)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Workload identity abstraction for HPC systems. This crate provides traits for obtaining mTLS workload identity from multiple sources (SPIRE, self-signed CA, bootstrap cert) with cascading provider selection and dual-channel certificate rotation.

This crate enables multiple applications (like [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice)) to share common identity infrastructure while implementing their own provider backends independently.

## Features

- **Provider Abstraction**: `IdentityProvider` trait for SPIRE, self-signed CA, and bootstrap cert sources
- **Cascading Selection**: `IdentityCascade` tries providers in priority order (SPIRE first, bootstrap last)
- **Certificate Rotation**: `CertRotator` trait for dual-channel swap without interrupting in-flight operations
- **Source Tracking**: `IdentitySource` enum tracks identity provenance for audit
- **Security**: Private keys redacted in `Debug` output, never logged or transmitted

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hpc-identity = "2026.1"
```

Or to use the latest development version from git:

```toml
[dependencies]
hpc-identity = { git = "https://github.com/witlox/hpc-core" }
```

## Usage

### 1. Implement an IdentityProvider

Each application implements providers for the identity sources available in its environment:

```rust
use hpc_identity::{IdentityProvider, IdentitySource, IdentityError, WorkloadIdentity};

struct MySpireProvider { /* SPIRE agent socket path, etc. */ }

#[async_trait::async_trait]
impl IdentityProvider for MySpireProvider {
    async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
        // Connect to SPIRE agent socket, obtain X.509 SVID
        todo!()
    }

    async fn is_available(&self) -> bool {
        // Check if SPIRE agent socket exists and is connectable
        todo!()
    }

    fn source_type(&self) -> IdentitySource {
        IdentitySource::Spire
    }
}
```

### 2. Use IdentityCascade

The cascade tries providers in order until one succeeds:

```rust
use hpc_identity::IdentityCascade;

let cascade = IdentityCascade::new(vec![
    Box::new(spire_provider),       // Try SPIRE first
    Box::new(self_signed_provider), // Fall back to journal-signed cert
    Box::new(bootstrap_provider),   // Last resort: bootstrap cert from image
]);

let identity = cascade.get_identity().await?;
println!("Identity acquired from: {:?}", identity.source);
// Use identity.cert_chain_pem + identity.private_key_pem for mTLS
```

### 3. Certificate Rotation

```rust
use hpc_identity::CertRotator;

struct MyRotator { /* active/passive gRPC channels */ }

#[async_trait::async_trait]
impl CertRotator for MyRotator {
    async fn rotate(&self, new_identity: WorkloadIdentity) -> Result<(), IdentityError> {
        // 1. Build passive gRPC channel with new cert
        // 2. Health-check passive channel
        // 3. Atomically swap passive → active
        // 4. Old channel drains in-flight operations
        todo!()
    }
}
```

### 4. Check Identity Validity

```rust
use chrono::Utc;

let identity = cascade.get_identity().await?;

if !identity.is_valid() {
    // Identity has expired — acquire a new one
}

if identity.should_renew(issued_at) {
    // Less than 1/3 of lifetime remaining — time to renew
    let new_identity = cascade.get_identity().await?;
    rotator.rotate(new_identity).await?;
}
```

## Architecture

### What's Provided (Shared Contract)

| Component | Description |
|-----------|-------------|
| `IdentityProvider` trait | Obtain workload identity from any source |
| `CertRotator` trait | Dual-channel certificate rotation pattern |
| `IdentityCascade` | Try providers in priority order |
| `WorkloadIdentity` | Source-agnostic cert + key + trust bundle |
| `IdentitySource` enum | Spire / SelfSigned / Bootstrap provenance |
| Provider configs | `SpireConfig`, `SelfSignedConfig`, `BootstrapConfig` |

### What You Provide (Application-Specific)

| Component | Description |
|-----------|-------------|
| `IdentityProvider` impls | Your SPIRE client, CSR signing client, cert file reader |
| `CertRotator` impl | Your gRPC channel swap logic |
| Provider selection | Your cascade ordering based on deployment environment |

### Identity Sources

| Source | When used | Rotation |
|--------|-----------|----------|
| **SPIRE** | Primary — HPE Cray infrastructure with SPIRE agent | SPIRE manages rotation automatically |
| **SelfSigned** | Fallback — no SPIRE deployed, journal/quorum signs CSRs | Agent-driven renewal at 2/3 of cert lifetime |
| **Bootstrap** | Initial — first boot before any provider is reachable | Temporary; replaced by SPIRE or SelfSigned ASAP |

### Design Principles

- **No hard SPIRE dependency** — system works with any provider, or just bootstrap
- **Private keys never leave the process** — generated locally, never transmitted
- **Rotation without interruption** — dual-channel swap preserves in-flight operations
- **Audit provenance** — `IdentitySource` tracks which provider issued each identity
