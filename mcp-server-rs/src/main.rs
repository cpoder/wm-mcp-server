mod is_client;
mod server;

use is_client::ISClient;
use rmcp::{ServiceExt, transport::stdio};
use server::WmServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr -- stdout is the MCP JSON-RPC transport
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let base_url = std::env::var("WM_IS_URL").unwrap_or_else(|_| "http://localhost:5555".into());
    let username = std::env::var("WM_IS_USER").unwrap_or_else(|_| "Administrator".into());
    let password = std::env::var("WM_IS_PASSWORD").unwrap_or_else(|_| "manage".into());
    let timeout: u64 = std::env::var("WM_IS_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    tracing::info!("Starting webMethods IS MCP server (target: {})", base_url);

    let client = ISClient::new(&base_url, &username, &password, timeout);
    let service = WmServer::new(client).serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
