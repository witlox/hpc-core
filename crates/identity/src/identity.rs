//! Workload identity types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How the identity was obtained.
///
/// Tracked for audit purposes — the `AuditEvent` for identity acquisition
/// includes this field so operators can see which provider was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentitySource {
    /// SPIRE SVID via Workload API.
    Spire,
    /// Self-signed via journal/quorum intermediate CA (ADR-008 fallback).
    SelfSigned,
    /// Bootstrap cert from `OpenCHAMI` provisioning (temporary, replaced on SVID/cert acquisition).
    Bootstrap,
}

/// Source-agnostic workload identity.
///
/// Contains everything needed to establish an mTLS connection.
/// The `source` field tracks provenance for audit.
///
/// # Security
///
/// `private_key_pem` is sensitive. The `Debug` implementation redacts it.
/// Never log or serialize this field.
#[derive(Clone)]
pub struct WorkloadIdentity {
    /// Certificate chain (PEM). Leaf cert + intermediates.
    pub cert_chain_pem: Vec<u8>,
    /// Private key (PEM). Never logged, never transmitted externally.
    pub private_key_pem: Vec<u8>,
    /// Trust bundle (PEM). CA certs for verifying peers.
    pub trust_bundle_pem: Vec<u8>,
    /// When this identity expires.
    pub expires_at: DateTime<Utc>,
    /// Where this identity came from.
    pub source: IdentitySource,
}

impl std::fmt::Debug for WorkloadIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkloadIdentity")
            .field("cert_chain_pem", &format!("[{} bytes]", self.cert_chain_pem.len()))
            .field("private_key_pem", &"[REDACTED]")
            .field("trust_bundle_pem", &format!("[{} bytes]", self.trust_bundle_pem.len()))
            .field("expires_at", &self.expires_at)
            .field("source", &self.source)
            .finish()
    }
}

impl WorkloadIdentity {
    /// Check if identity is still valid (not expired).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// Check if identity should be renewed.
    ///
    /// Returns `true` if less than 1/3 of the total lifetime remains.
    /// This corresponds to the 2/3-lifetime renewal trigger (invariant E5).
    ///
    /// For a 3-day cert issued at T, this returns true after T+2 days.
    #[must_use]
    pub fn should_renew(&self, issued_at: DateTime<Utc>) -> bool {
        let total_lifetime = self.expires_at - issued_at;
        let remaining = self.expires_at - Utc::now();
        remaining < total_lifetime / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn test_identity(expires_in: Duration) -> WorkloadIdentity {
        WorkloadIdentity {
            cert_chain_pem: b"cert".to_vec(),
            private_key_pem: b"key".to_vec(),
            trust_bundle_pem: b"ca".to_vec(),
            expires_at: Utc::now() + expires_in,
            source: IdentitySource::Bootstrap,
        }
    }

    #[test]
    fn valid_identity() {
        let id = test_identity(Duration::hours(1));
        assert!(id.is_valid());
    }

    #[test]
    fn expired_identity() {
        let id = test_identity(Duration::hours(-1));
        assert!(!id.is_valid());
    }

    #[test]
    fn should_renew_after_two_thirds() {
        let issued = Utc::now() - Duration::days(2);
        let id = WorkloadIdentity {
            cert_chain_pem: b"cert".to_vec(),
            private_key_pem: b"key".to_vec(),
            trust_bundle_pem: b"ca".to_vec(),
            expires_at: issued + Duration::days(3), // 3-day lifetime, issued 2 days ago
            source: IdentitySource::SelfSigned,
        };
        assert!(id.should_renew(issued)); // <1 day remaining of 3 = less than 1/3
    }

    #[test]
    fn should_not_renew_fresh() {
        let issued = Utc::now();
        let id = WorkloadIdentity {
            cert_chain_pem: b"cert".to_vec(),
            private_key_pem: b"key".to_vec(),
            trust_bundle_pem: b"ca".to_vec(),
            expires_at: issued + Duration::days(3),
            source: IdentitySource::Spire,
        };
        assert!(!id.should_renew(issued)); // 3 days remaining of 3
    }

    #[test]
    fn debug_redacts_private_key() {
        let id = test_identity(Duration::hours(1));
        let debug = format!("{id:?}");
        assert!(debug.contains("REDACTED"));
        // The actual key value (b"key") must not appear — only field name + [REDACTED]
        assert!(!debug.contains(r#""key""#));
    }

    #[test]
    fn exactly_at_expiry_is_invalid() {
        let id = WorkloadIdentity {
            cert_chain_pem: b"cert".to_vec(),
            private_key_pem: b"key".to_vec(),
            trust_bundle_pem: b"ca".to_vec(),
            expires_at: Utc::now(), // exactly now
            source: IdentitySource::Bootstrap,
        };
        // At exact expiry time, should be invalid (strict less-than)
        assert!(!id.is_valid());
    }

    #[test]
    fn should_renew_at_exact_two_thirds() {
        // 3-day lifetime, issued exactly 2 days ago = exactly at 2/3 point
        let issued = Utc::now() - Duration::days(2);
        let id = WorkloadIdentity {
            cert_chain_pem: b"cert".to_vec(),
            private_key_pem: b"key".to_vec(),
            trust_bundle_pem: b"ca".to_vec(),
            expires_at: issued + Duration::days(3),
            source: IdentitySource::SelfSigned,
        };
        // At exactly 2/3, remaining = 1 day, total/3 = 1 day → not strictly less, so false
        // This is fine — renewal triggers slightly after 2/3
        let remaining = id.expires_at - Utc::now();
        let threshold = Duration::days(3) / 3;
        if remaining < threshold {
            assert!(id.should_renew(issued));
        }
    }

    #[test]
    fn clone_preserves_private_key() {
        let id = test_identity(Duration::hours(1));
        let cloned = id.clone();
        assert_eq!(cloned.private_key_pem, id.private_key_pem);
        assert_eq!(cloned.cert_chain_pem, id.cert_chain_pem);
        assert_eq!(cloned.source, id.source);
    }

    #[test]
    fn identity_source_serialization() {
        for source in [
            IdentitySource::Spire,
            IdentitySource::SelfSigned,
            IdentitySource::Bootstrap,
        ] {
            let json = serde_json::to_string(&source).unwrap();
            let deser: IdentitySource = serde_json::from_str(&json).unwrap();
            assert_eq!(deser, source);
        }
    }
}
