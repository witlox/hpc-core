//! Identity error types.

/// Errors from identity operations.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("SPIRE agent unavailable: {reason}")]
    SpireUnavailable { reason: String },

    #[error("CSR signing failed: {reason}")]
    CsrSigningFailed { reason: String },

    #[error("bootstrap identity not found: {path}")]
    BootstrapNotFound { path: String },

    #[error("identity expired")]
    Expired,

    #[error("rotation failed: {reason}")]
    RotationFailed { reason: String },

    #[error("no identity provider available")]
    NoProviderAvailable,

    #[error("identity I/O error: {0}")]
    Io(#[from] std::io::Error),
}
