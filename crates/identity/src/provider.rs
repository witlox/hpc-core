//! Identity provider trait.

use crate::{IdentityError, IdentitySource, WorkloadIdentity};

/// Trait for obtaining workload identity.
///
/// Implementations:
/// - `SpireProvider` — connects to SPIRE agent socket, obtains X.509 SVID
/// - `SelfSignedProvider` — generates keypair + CSR, submits to journal/quorum CA
/// - `StaticProvider` — reads bootstrap cert from filesystem
///
/// # Contract
///
/// - `get_identity()` must return a valid `WorkloadIdentity` or an error.
/// - Implementations handle their own retry logic.
/// - Private keys are generated locally, never transmitted.
/// - The `source` field in returned identity must accurately reflect provenance.
#[async_trait::async_trait]
pub trait IdentityProvider: Send + Sync {
    /// Obtain a workload identity.
    ///
    /// May involve network calls (SPIRE socket, journal CSR signing endpoint).
    /// Implementations should have reasonable timeouts.
    async fn get_identity(&self) -> Result<WorkloadIdentity, IdentityError>;

    /// Check if this provider is currently available.
    ///
    /// For `SpireProvider`: checks if SPIRE agent socket exists and is connectable.
    /// For `SelfSignedProvider`: checks if signing endpoint is reachable.
    /// For `StaticProvider`: checks if cert files exist and are readable.
    ///
    /// This is a fast check — does not attempt full identity acquisition.
    async fn is_available(&self) -> bool;

    /// The source type this provider produces.
    fn source_type(&self) -> IdentitySource;
}
