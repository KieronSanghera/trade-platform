pub mod trade {
    tonic::include_proto!("trade");
}

pub mod config; // Runtime configuration loader
pub mod error; // Service-specific error types
pub mod grpc; // gRPC service implementation
pub mod health; // Health and Ready endpoints
pub mod models; // Domain models
pub mod producer; // Producer abstraction layer
