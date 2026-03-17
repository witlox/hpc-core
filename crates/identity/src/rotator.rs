//! Certificate rotation trait.

use crate::{IdentityError, WorkloadIdentity};

/// Trait for certificate rotation.
///
/// Default implementation pattern: dual-channel swap (ADR-008, E6):
/// 1. Build passive gRPC channel with new cert
/// 2. Health-check passive channel
/// 3. Atomically swap passive → active
/// 4. Old channel drains in-flight operations
///
/// # Contract
///
/// - `rotate()` must not interrupt in-flight operations
/// - Old channel completes pending RPCs before being dropped
/// - If rotation fails, active channel continues unchanged
#[async_trait::async_trait]
pub trait CertRotator: Send + Sync {
    /// Rotate to a new identity.
    ///
    /// Builds a passive connection with the new identity, health-checks it,
    /// then atomically swaps with the active connection.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError::RotationFailed`] if the new identity cannot
    /// establish a valid connection. The active connection is not affected.
    async fn rotate(&self, new_identity: WorkloadIdentity) -> Result<(), IdentityError>;
}
