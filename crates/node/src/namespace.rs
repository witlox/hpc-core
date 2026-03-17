//! Namespace handoff protocol.
//!
//! Defines the protocol for passing Linux namespace file descriptors between
//! pact-agent (provider) and lattice-node-agent (consumer) via unix socket
//! with `SCM_RIGHTS`.

use serde::{Deserialize, Serialize};

/// Well-known socket path for namespace handoff.
///
/// pact-agent listens on this socket. lattice-node-agent connects to request
/// namespaces for allocations.
pub const HANDOFF_SOCKET_PATH: &str = "/run/pact/handoff.sock";

/// Namespace types that can be created and handed off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NamespaceType {
    /// PID namespace — isolated process ID space per allocation.
    Pid,
    /// Network namespace — isolated network stack per allocation.
    Net,
    /// Mount namespace — isolated mount table per allocation.
    Mount,
}

/// Request from lattice to pact for allocation namespaces.
///
/// Sent over the handoff unix socket as a framed message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceRequest {
    /// Allocation identifier (from lattice scheduler).
    pub allocation_id: String,
    /// Which namespaces to create.
    pub namespaces: Vec<NamespaceType>,
    /// Optional uenv image to mount inside the mount namespace.
    pub uenv_image: Option<String>,
}

/// Response from pact to lattice with namespace metadata.
///
/// The actual namespace file descriptors are passed via `SCM_RIGHTS`
/// ancillary data on the unix socket, in the same order as
/// `requested_types`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceResponse {
    /// Allocation identifier (echoed from request).
    pub allocation_id: String,
    /// Namespace types in the order their FDs are attached via `SCM_RIGHTS`.
    pub fd_types: Vec<NamespaceType>,
    /// Mount point for uenv bind-mount inside the mount namespace (if requested).
    pub uenv_mount_path: Option<String>,
}

/// Notification that an allocation has ended.
///
/// Sent when pact detects that all processes in the allocation's cgroup
/// have exited (WI5: cgroup-empty detection). Lattice can also send this
/// proactively if it knows the allocation ended.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEnded {
    /// Allocation identifier.
    pub allocation_id: String,
}

/// Trait for the namespace handoff provider (pact-agent implements).
///
/// Creates Linux namespaces for allocations and makes their FDs available
/// for passing to lattice via the unix socket.
pub trait NamespaceProvider: Send + Sync {
    /// Create namespaces for an allocation.
    ///
    /// Returns metadata about the created namespaces. The actual FDs are
    /// made available for the handoff socket to send via `SCM_RIGHTS`.
    ///
    /// # Errors
    ///
    /// Returns [`NamespaceError::CreationFailed`] if `unshare(2)` or namespace
    /// setup fails.
    fn create_namespaces(
        &self,
        request: &NamespaceRequest,
    ) -> Result<NamespaceResponse, NamespaceError>;

    /// Release namespaces for a completed allocation.
    ///
    /// Cleans up namespace FDs and any associated resources (bind-mounts).
    fn release_namespaces(&self, allocation_id: &str) -> Result<(), NamespaceError>;
}

/// Trait for the namespace handoff consumer (lattice-node-agent implements).
///
/// Requests namespaces from the provider (pact). When the provider is
/// unavailable, falls back to self-service namespace creation (WI4, F27).
pub trait NamespaceConsumer: Send + Sync {
    /// Request namespaces from the provider.
    ///
    /// If the provider is unavailable (handoff socket not reachable),
    /// implementations should fall back to creating their own namespaces
    /// using the same conventions (WI4).
    fn request_namespaces(
        &self,
        request: &NamespaceRequest,
    ) -> Result<NamespaceResponse, NamespaceError>;
}

/// Errors from namespace operations.
#[derive(Debug, thiserror::Error)]
pub enum NamespaceError {
    #[error("handoff socket unavailable: {reason}")]
    SocketUnavailable { reason: String },

    #[error("namespace creation failed: {reason}")]
    CreationFailed { reason: String },

    #[error("allocation not found: {allocation_id}")]
    AllocationNotFound { allocation_id: String },

    #[error("FD passing failed: {0}")]
    FdPassing(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_request_serialization() {
        let req = NamespaceRequest {
            allocation_id: "alloc-42".to_string(),
            namespaces: vec![NamespaceType::Pid, NamespaceType::Net, NamespaceType::Mount],
            uenv_image: Some("pytorch-2.5.sqfs".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deser: NamespaceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.allocation_id, "alloc-42");
        assert_eq!(deser.namespaces.len(), 3);
        assert_eq!(deser.uenv_image.as_deref(), Some("pytorch-2.5.sqfs"));
    }

    #[test]
    fn namespace_response_fd_order() {
        let resp = NamespaceResponse {
            allocation_id: "alloc-42".to_string(),
            fd_types: vec![NamespaceType::Pid, NamespaceType::Net, NamespaceType::Mount],
            uenv_mount_path: Some("/run/pact/uenv/pytorch-2.5".to_string()),
        };
        assert_eq!(resp.fd_types[0], NamespaceType::Pid);
        assert_eq!(resp.fd_types[1], NamespaceType::Net);
        assert_eq!(resp.fd_types[2], NamespaceType::Mount);
    }
}
