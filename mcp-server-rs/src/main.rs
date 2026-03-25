mod config;
mod is_client;
mod prompts;
mod server;

use config::AppConfig;
use is_client::ISClient;
use rmcp::{ServiceExt, transport::stdio};
use server::WmServer;
use std::collections::HashMap;
use std::sync::Arc;

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

    let config = AppConfig::load().map_err(|e| anyhow::anyhow!(e))?;

    let mut clients = HashMap::new();
    for (name, inst) in &config.instances {
        tracing::info!("Registering IS instance '{}' at {}", name, inst.url);
        let client = ISClient::new(&inst.url, &inst.user, &inst.password, inst.timeout);
        clients.insert(name.clone(), Arc::new(client));
    }

    tracing::info!(
        "Starting webMethods IS MCP server ({} instance(s), default: '{}')",
        clients.len(),
        config.default_instance,
    );

    let service = WmServer::new(clients, config.default_instance)
        .serve(stdio())
        .await?;
    service.waiting().await?;

    Ok(())
}
