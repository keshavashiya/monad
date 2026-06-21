//! Shared data structures used by firmware and daemon crates.
//! Re-exported vault types live in the firmware crate directly for now.
//! This crate is reserved for cross-crate types (e.g. daemon protocol).

pub mod types {
    use serde::{Deserialize, Serialize};

    /// Request/response types for daemon HTTP API
    #[derive(Deserialize)]
    pub struct ExecuteRequest {
        pub input: String,
    }

    #[derive(Serialize)]
    pub struct ExecuteResponse {
        pub output: String,
    }
}
