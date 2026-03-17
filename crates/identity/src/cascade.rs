//! Cascading identity provider.

use tracing::{info, warn};

use crate::{IdentityError, IdentityProvider, WorkloadIdentity};

/// Cascading identity provider.
///
/// Tries providers in order until one succeeds. Designed for the
/// SPIRE → self-signed → bootstrap fallback chain (PB5: no hard SPIRE dependency).
///
/// # Behavior
///
/// - Tries each provider's `is_available()` first (fast check)
/// - If available, calls `get_identity()`
/// - On success, returns immediately with that identity
/// - On failure, logs warning and tries next provider
/// - If all fail, returns the last error
///
/// # Note on provider upgrades
///
/// The cascade does NOT retry earlier providers within the same call.
/// If bootstrap succeeds first, the caller gets a bootstrap identity.
/// Upgrading to SPIRE when it becomes available later is handled by
/// periodic renewal in the caller, not by the cascade.
pub struct IdentityCascade {
    providers: Vec<Box<dyn IdentityProvider>>,
}

impl IdentityCascade {
    /// Create a new cascade with providers in priority order.
    ///
    /// The first provider is tried first (highest priority).
    /// Typical order: SPIRE → self-signed → bootstrap.
    #[must_use]
    pub fn new(providers: Vec<Box<dyn IdentityProvider>>) -> Self {
        Self { providers }
    }

    /// Get identity from the first available provider.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError::NoProviderAvailable`] if no providers are configured.
    /// Returns the last provider's error if all providers fail.
    pub async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
        if self.providers.is_empty() {
            return Err(IdentityError::NoProviderAvailable);
        }

        let mut last_error = None;

        for provider in &self.providers {
            let source = provider.source_type();

            if !provider.is_available().await {
                info!(?source, "identity provider not available, skipping");
                continue;
            }

            match provider.get_identity().await {
                Ok(identity) => {
                    info!(?source, "identity acquired successfully");
                    return Ok(identity);
                }
                Err(e) => {
                    warn!(?source, error = %e, "identity provider failed, trying next");
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(IdentityError::NoProviderAvailable))
    }

    /// Returns the number of configured providers.
    #[must_use]
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IdentitySource;
    use chrono::{Duration, Utc};

    struct AlwaysAvailable {
        source: IdentitySource,
    }

    #[async_trait::async_trait]
    impl IdentityProvider for AlwaysAvailable {
        async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
            Ok(WorkloadIdentity {
                cert_chain_pem: b"cert".to_vec(),
                private_key_pem: b"key".to_vec(),
                trust_bundle_pem: b"ca".to_vec(),
                expires_at: Utc::now() + Duration::hours(1),
                source: self.source,
            })
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn source_type(&self) -> IdentitySource {
            self.source
        }
    }

    struct NeverAvailable;

    #[async_trait::async_trait]
    impl IdentityProvider for NeverAvailable {
        async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
            Err(IdentityError::SpireUnavailable {
                reason: "test".to_string(),
            })
        }

        async fn is_available(&self) -> bool {
            false
        }

        fn source_type(&self) -> IdentitySource {
            IdentitySource::Spire
        }
    }

    struct AvailableButFails;

    #[async_trait::async_trait]
    impl IdentityProvider for AvailableButFails {
        async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
            Err(IdentityError::CsrSigningFailed {
                reason: "test failure".to_string(),
            })
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn source_type(&self) -> IdentitySource {
            IdentitySource::SelfSigned
        }
    }

    #[tokio::test]
    async fn cascade_uses_first_available() {
        let cascade = IdentityCascade::new(vec![
            Box::new(NeverAvailable),
            Box::new(AlwaysAvailable {
                source: IdentitySource::SelfSigned,
            }),
            Box::new(AlwaysAvailable {
                source: IdentitySource::Bootstrap,
            }),
        ]);

        let id = cascade.get_identity().await.unwrap();
        assert_eq!(id.source, IdentitySource::SelfSigned);
    }

    #[tokio::test]
    async fn cascade_skips_failed_provider() {
        let cascade = IdentityCascade::new(vec![
            Box::new(AvailableButFails),
            Box::new(AlwaysAvailable {
                source: IdentitySource::Bootstrap,
            }),
        ]);

        let id = cascade.get_identity().await.unwrap();
        assert_eq!(id.source, IdentitySource::Bootstrap);
    }

    #[tokio::test]
    async fn cascade_empty_returns_error() {
        let cascade = IdentityCascade::new(vec![]);
        let err = cascade.get_identity().await.unwrap_err();
        assert!(matches!(err, IdentityError::NoProviderAvailable));
    }

    #[tokio::test]
    async fn cascade_all_fail_returns_last_error() {
        let cascade = IdentityCascade::new(vec![
            Box::new(NeverAvailable),
            Box::new(AvailableButFails),
        ]);

        let err = cascade.get_identity().await.unwrap_err();
        assert!(matches!(err, IdentityError::CsrSigningFailed { .. }));
    }

    #[tokio::test]
    async fn cascade_returns_expired_identity_if_provider_does() {
        // Provider returns an already-expired identity — cascade doesn't validate,
        // that's the caller's responsibility
        struct ExpiringProvider;

        #[async_trait::async_trait]
        impl IdentityProvider for ExpiringProvider {
            async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError> {
                Ok(WorkloadIdentity {
                    cert_chain_pem: b"cert".to_vec(),
                    private_key_pem: b"key".to_vec(),
                    trust_bundle_pem: b"ca".to_vec(),
                    expires_at: Utc::now() - Duration::hours(1), // already expired
                    source: IdentitySource::Bootstrap,
                })
            }
            async fn is_available(&self) -> bool {
                true
            }
            fn source_type(&self) -> IdentitySource {
                IdentitySource::Bootstrap
            }
        }

        let cascade = IdentityCascade::new(vec![Box::new(ExpiringProvider)]);
        let id = cascade.get_identity().await.unwrap();
        // Cascade returns it — caller must check is_valid()
        assert!(!id.is_valid());
    }

    #[tokio::test]
    async fn cascade_provider_count() {
        let cascade = IdentityCascade::new(vec![
            Box::new(NeverAvailable),
            Box::new(AlwaysAvailable {
                source: IdentitySource::Bootstrap,
            }),
        ]);
        assert_eq!(cascade.provider_count(), 2);
    }

    #[tokio::test]
    async fn cascade_prefers_first_available_provider() {
        // Both available — first one wins (SPIRE before Bootstrap)
        let cascade = IdentityCascade::new(vec![
            Box::new(AlwaysAvailable {
                source: IdentitySource::Spire,
            }),
            Box::new(AlwaysAvailable {
                source: IdentitySource::Bootstrap,
            }),
        ]);

        let id = cascade.get_identity().await.unwrap();
        assert_eq!(id.source, IdentitySource::Spire);
    }
}
