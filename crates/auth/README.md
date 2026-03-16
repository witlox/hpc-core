# hpc-auth

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-auth.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-auth.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-auth.svg)](https://crates.io/crates/hpc-auth)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

OAuth2/OIDC authentication client for HPC systems. This crate provides a complete auth workflow including token acquisition, caching, refresh, and revocation with support for multiple OAuth2 flows.

This crate enables multiple applications (like [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice)) to share common authentication infrastructure while configuring their own IdP settings and flow preferences.

## Features

- **Multiple OAuth2 Flows**: Authorization Code with PKCE, Device Code, Client Credentials, Manual Paste (SSH/headless)
- **Per-Server Token Caching**: File-based, isolated by server URL, with Unix permission validation (0600)
- **Automatic Token Refresh**: Transparent re-authentication on expiration
- **OIDC Discovery**: Automatic IdP configuration via `.well-known/openid-configuration`
- **Discovery Caching**: In-memory cache with 1-hour TTL and degraded-mode fallback
- **Security-First**: Fail-closed cache validation, secret redaction in logs, CSRF protection

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hpc-auth = "2026.1"
```

Or to use the latest development version from git:

```toml
[dependencies]
hpc-auth = { git = "https://github.com/witlox/hpc-core" }
```

## Usage

### 1. Create an AuthClient

```rust
use hpc_auth::{AuthClient, AuthClientConfig, PermissionMode};
use std::time::Duration;

let config = AuthClientConfig {
    server_url: "https://journal.example.com:9443".to_string(),
    app_name: "my-app".to_string(),
    permission_mode: PermissionMode::Strict,
    idp_override: None,
    flow_override: None,
    timeout: Duration::from_secs(30),
};

let client = AuthClient::new(config);
```

### 2. Login and Get Tokens

```rust
// Interactive login — discovers IdP, selects best flow, acquires tokens
let token_set = client.login().await?;

// Get a valid token (returns cached or refreshes if expired)
let token_set = client.get_token().await?;

// Check login status without refreshing
let logged_in = client.is_logged_in().await;
```

### 3. Logout

```rust
// Clears local cache first, then revokes at IdP (best-effort)
client.logout().await?;
```

## Architecture

### What's Provided

| Component | Description |
|-----------|-------------|
| `AuthClient` | Main entry point — login, logout, token refresh orchestration |
| `TokenCache` | File-based per-server token storage with permission validation |
| `DiscoveryCache` | In-memory OIDC discovery cache with TTL and degraded-mode fallback |
| OAuth2 Flows | Authorization Code PKCE, Device Code, Client Credentials, Manual Paste |

### What You Provide (Application-Specific)

| Component | Description |
|-----------|-------------|
| `AuthClientConfig` | Server URL, app name, permission mode, timeouts |
| `IdpConfig` (optional) | Override IdP settings to skip server discovery |
| `OAuthFlow` (optional) | Force a specific OAuth2 flow |

### Token Cache Layout

```
~/.config/{app_name}/auth/
├── tokens-{server_hash}.json    # Per-server token cache (mode 0600)
└── default-server.json          # Default server URL pointer
```
