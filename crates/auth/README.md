# hpc-auth

[![CI](https://github.com/witlox/hpc-core/actions/workflows/ci-auth.yml/badge.svg)](https://github.com/witlox/hpc-core/actions/workflows/ci-auth.yml)
[![crates.io](https://img.shields.io/crates/v/hpc-auth.svg)](https://crates.io/crates/hpc-auth)
[![codecov](https://codecov.io/gh/witlox/hpc-core/graph/badge.svg?flag=auth)](https://codecov.io/gh/witlox/hpc-core)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

OAuth2/OIDC authentication client for HPC systems. Used by [Pact](https://github.com/witlox/pact) and [Lattice](https://github.com/witlox/lattice).

## Features

- **Multiple OAuth2 flows** — Authorization Code with PKCE, Device Code, Client Credentials, Manual Paste (SSH/headless)
- **Per-server token caching** — file-based, isolated by server URL, with Unix permission validation (0600)
- **Automatic token refresh** — transparent re-authentication on expiration
- **OIDC discovery** — automatic IdP configuration via `.well-known/openid-configuration`
- **Security-first** — fail-closed cache validation, secret redaction in logs, CSRF protection

## Installation

```toml
[dependencies]
hpc-auth = "2026.1"
```
