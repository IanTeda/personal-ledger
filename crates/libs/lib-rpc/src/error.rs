
pub type RpcResult<T> = std::result::Result<T, RpcError>;

/// Errors that can occur when using the utilities client.
#[derive(thiserror::Error, Debug)]
pub enum RpcError {
    /// Failed to connect to the gRPC service.
    #[error("connection error: {0}")]
    Connection(#[from] tonic::transport::Error),

    /// gRPC operation failed with a status error.
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    /// Internal client error (e.g., invalid state).
    #[error("client error: {0}")]
    Client(String),
}