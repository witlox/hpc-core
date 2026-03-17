//! Provider configuration types.

use serde::{Deserialize, Serialize};

/// Configuration for SPIRE provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpireConfig {
    /// Path to SPIRE agent Workload API socket.
    pub agent_socket: String,
    /// SPIFFE ID to request. `None` = auto-detect from attestation.
    pub spiffe_id: Option<String>,
    /// Timeout for SVID acquisition in seconds.
    pub timeout_seconds: u64,
}

impl Default for SpireConfig {
    fn default() -> Self {
        Self {
            agent_socket: "/run/spire/agent.sock".to_string(),
            spiffe_id: None,
            timeout_seconds: 30,
        }
    }
}

/// Configuration for self-signed provider (ADR-008 fallback).
///
/// Agent generates keypair + CSR, submits to journal/quorum CA for signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfSignedConfig {
    /// Journal/quorum endpoint for CSR signing (gRPC).
    pub signing_endpoint: String,
    /// Certificate lifetime in seconds. Default: 3 days (259200s).
    pub cert_lifetime_seconds: u64,
}

impl Default for SelfSignedConfig {
    fn default() -> Self {
        Self {
            signing_endpoint: String::new(),
            cert_lifetime_seconds: 259_200, // 3 days
        }
    }
}

/// Configuration for bootstrap/static provider.
///
/// Reads pre-provisioned certificate from the `SquashFS` image or tmpfs.
/// This identity is temporary — replaced by SPIRE SVID or self-signed cert.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct BootstrapConfig {
    /// Path to bootstrap certificate (PEM).
    pub cert_path: String,
    /// Path to bootstrap private key (PEM).
    pub key_path: String,
    /// Path to trust bundle (PEM).
    pub trust_bundle_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spire_config_defaults() {
        let config = SpireConfig::default();
        assert_eq!(config.agent_socket, "/run/spire/agent.sock");
        assert!(config.spiffe_id.is_none());
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn self_signed_config_defaults() {
        let config = SelfSignedConfig::default();
        assert!(config.signing_endpoint.is_empty());
        assert_eq!(config.cert_lifetime_seconds, 259_200);
    }

    #[test]
    fn config_serialization() {
        let config = SpireConfig {
            agent_socket: "/custom/spire.sock".to_string(),
            spiffe_id: Some("spiffe://example.com/pact-agent".to_string()),
            timeout_seconds: 60,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deser: SpireConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.agent_socket, "/custom/spire.sock");
        assert_eq!(
            deser.spiffe_id.as_deref(),
            Some("spiffe://example.com/pact-agent")
        );
    }
}
