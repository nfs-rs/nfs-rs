use nfs_rs::{NfsConfig, NfsServer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Print help and exit if --help is present
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("nfs-server: Minimal NFSv4.2 server in Rust\n\nUsage:\n    cargo run --bin nfs-server [--help]\n\nEnvironment variables:\n    NFS_BIND_ADDR   Bind address (default: 127.0.0.1)\n    NFS_PORT        Port to listen on (default: 20490)\n\nExample:\n    RUST_LOG=info NFS_BIND_ADDR=127.0.0.1 NFS_PORT=20490 cargo run --bin nfs-server\n");
        return Ok(());
    }

    // Allow simple configuration via environment variables for quick runs
    let mut cfg = NfsConfig::default();
    if let Ok(addr) = std::env::var("NFS_BIND_ADDR") { cfg.bind_addr = addr; }
    if let Ok(port) = std::env::var("NFS_PORT") { if let Ok(p) = port.parse() { cfg.port = p; } }
    let server = NfsServer::new(cfg).await?;
    server.run().await?;
    Ok(())
}
