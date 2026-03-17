# hpc-audit

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-audit.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-audit.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-audit.svg)](https://crates.io/crates/hpc-audit)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Shared audit event types and sink trait for HPC systems. This crate provides a universal `AuditEvent` format and `AuditSink` trait for consistent audit logging across distributed HPC infrastructure.

This crate enables multiple applications (like [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice)) to emit audit events in a shared format, enabling unified SIEM forwarding without runtime coupling between systems.

## Features

- **Universal Event Format**: `AuditEvent` with who, what, when, where, outcome, and structured metadata
- **Sink Trait**: Pluggable destinations (journal, file, SIEM, Loki) via `AuditSink`
- **Compliance Policies**: Configurable retention rules and required audit points (default and 7-year regulated)
- **Action Constants**: Well-known action strings for cross-system consistency
- **Test Utilities**: `MemoryAuditSink` (collects events in memory) and `NullAuditSink` (discards)
- **Thread-Safe**: `MemoryAuditSink` is safe for concurrent use from multiple threads

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hpc-audit = "2026.1"
```

Or to use the latest development version from git:

```toml
[dependencies]
hpc-audit = { git = "https://github.com/witlox/hpc-core" }
```

## Usage

### 1. Emit Audit Events

```rust
use hpc_audit::*;
use chrono::Utc;

let event = AuditEvent {
    id: "evt-001".to_string(),
    timestamp: Utc::now(),
    principal: AuditPrincipal {
        identity: "admin@example.com".to_string(),
        principal_type: PrincipalType::Human,
        role: "pact-ops-ml-training".to_string(),
    },
    action: actions::SERVICE_RESTART.to_string(),
    scope: AuditScope::node_vcluster("node-001", "ml-training"),
    outcome: AuditOutcome::Success,
    detail: "Restarted nvidia-persistenced after GPU driver update".to_string(),
    metadata: serde_json::json!({"service": "nvidia-persistenced", "restart_count": 1}),
    source: AuditSource::PactAgent,
};

// Emit to any AuditSink implementation
audit_sink.emit(event);
```

### 2. Implement an AuditSink

```rust
use hpc_audit::{AuditSink, AuditEvent, AuditError};

struct MyJournalSink { /* ... */ }

impl AuditSink for MyJournalSink {
    fn emit(&self, event: AuditEvent) {
        // Buffer event for async flush to journal
        // CONTRACT: must not block the caller
    }

    fn flush(&self) -> Result<(), AuditError> {
        // Flush buffered events to journal
        Ok(())
    }
}
```

### 3. Use Compliance Policies

```rust
use hpc_audit::{CompliancePolicy, actions};

// Default: 1-year retention, critical actions only
let default = CompliancePolicy::default();
assert!(default.is_required(actions::SERVICE_CRASH));
assert!(!default.is_required(actions::SERVICE_START));

// Regulated: 7-year retention, all actions logged
let regulated = CompliancePolicy::regulated();
assert!(regulated.is_required(actions::SERVICE_START));
assert!(regulated.log_all_access);
```

### 4. Testing with MemoryAuditSink

```rust
use hpc_audit::{MemoryAuditSink, AuditSink};

let sink = MemoryAuditSink::new();
sink.emit(my_event);

assert_eq!(sink.len(), 1);
let events = sink.events();
assert_eq!(events[0].action, "service.restart");
```

## Architecture

### What's Provided (Shared Contract)

| Component | Description |
|-----------|-------------|
| `AuditEvent` | Universal event type with structured metadata |
| `AuditSink` trait | Pluggable destination interface (non-blocking emit + flush) |
| `CompliancePolicy` | Retention rules and required audit points |
| `actions` module | Well-known action string constants |
| `MemoryAuditSink` | In-memory sink for testing (thread-safe) |
| `NullAuditSink` | No-op sink for testing or disabled audit |

### What You Provide (Application-Specific)

| Component | Description |
|-----------|-------------|
| `AuditSink` impl | Your destination (journal append, file write, SIEM forward, Loki push) |
| Event emission | Your code calls `sink.emit(event)` at audit points |
| Compliance config | Your vCluster policy selects default or regulated compliance |

### Design Principles

- **Loose coupling, high coherence** — each system owns its audit log, shared format for SIEM
- **Non-blocking emit** — `AuditSink::emit()` must not block; buffer internally
- **Audit trail continuity** — sinks must not silently drop events
- **Source tracking** — `AuditSource` enum distinguishes which system emitted each event
