use nfs_rs::{NfsConfig, NfsServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = NfsConfig::default();
    let server = NfsServer::new(cfg).await?;
    server.run().await?;
    Ok(())
}
