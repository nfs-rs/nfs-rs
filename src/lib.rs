//! # NFS v4.2 Server Implementation in Rust
//!
//! This crate provides a high-performance Network File System (NFS) version 4.2 server
//! implementation written in Rust, using tokio for async I/O and serde for serialization.
//!
//! ## Features
//!
//! - **NFSv4.2 Protocol Support** - Full implementation of NFSv4.2 specification (RFC 7862)
//! - **High Performance** - Built with Rust's zero-cost abstractions and memory safety
//! - **Cross-Platform** - Runs on Linux, macOS, and Windows
//! - **Async I/O** - Non-blocking operations using Tokio runtime
//! - **Configurable** - Flexible configuration options for various use cases
//! - **Logging** - Comprehensive logging with configurable levels
//!
//! ## NFSv4.2 Features
//!
//! - Server-Side Copy operations
//! - Application I/O hints
//! - Sparse file support
//! - Space reservation
//! - Application Data Block (ADB) support
//! - Labeled NFS for MAC security models
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nfs_rs::{NfsServer, NfsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = NfsConfig::default();
//!     let server = NfsServer::new(config).await?;
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod proto;
pub mod rpc;
pub mod server;
pub mod vfs;
pub mod xdr;

pub use config::NfsConfig;
pub use error::{NfsError, NfsResult};
pub use server::NfsServer;

/// NFS Version 4.2 constants
pub mod constants {
    /// NFS Program number
    pub const NFS_PROGRAM: u32 = 100003;

    /// NFS Version 4.2
    pub const NFS_VERSION: u32 = 4;

    /// NFS Minor Version 2
    pub const NFS_MINOR_VERSION: u32 = 2;

    /// Default NFS port
    pub const NFS_PORT: u16 = 2049;

    /// Maximum file handle size
    pub const NFS4_FHSIZE: u32 = 128;

    /// Maximum component name length
    pub const NFS4_MAXNAMLEN: u32 = 255;

    /// Maximum path name length
    pub const NFS4_MAXPATHLEN: u32 = 4096;
}
