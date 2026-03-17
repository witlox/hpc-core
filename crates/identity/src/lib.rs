//! # hpc-identity
//!
//! Workload identity abstraction for HPC systems.
//!
//! Provides traits for obtaining mTLS workload identity from multiple sources
//! (SPIRE, self-signed CA, bootstrap cert) with cascading provider selection
//! and dual-channel certificate rotation.
//!
//! ## Usage
//!
//! Both pact-agent and lattice-node-agent use [`IdentityCascade`] to obtain
//! their workload identity:
//!
//! ```ignore
//! use hpc_identity::*;
//!
//! let cascade = IdentityCascade::new(vec![
//!     Box::new(spire_provider),      // Try SPIRE first
//!     Box::new(self_signed_provider), // Fall back to journal-signed
//!     Box::new(bootstrap_provider),   // Last resort: bootstrap cert
//! ]);
//!
//! let identity = cascade.get_identity().await?;
//! // identity.source tells you which provider succeeded
//! ```

mod cascade;
mod config;
mod error;
mod identity;
mod provider;
mod rotator;

pub use cascade::IdentityCascade;
pub use config::{BootstrapConfig, SelfSignedConfig, SpireConfig};
pub use error::IdentityError;
pub use identity::{IdentitySource, WorkloadIdentity};
pub use provider::IdentityProvider;
pub use rotator::CertRotator;
