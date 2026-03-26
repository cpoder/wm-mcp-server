mod client;
mod config;
mod params;
mod prompts;
mod resources;
mod scopes;
mod server;

use client::ISClient;
use config::AppConfig;
use rmcp::{ServiceExt, transport::stdio};
use server::WmServer;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr -- stdout is the MCP JSON-RPC transport (stdio mode)
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

    // Parse CLI args: --http <port> for HTTP mode, default is stdio
    let args: Vec<String> = std::env::args().collect();
    let http_port = args
        .iter()
        .position(|a| a == "--http")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok());

    if !config.scopes.is_empty() {
        tracing::info!("Tool scopes: {:?}", config.scopes);
    }

    if let Some(port) = http_port {
        run_http(clients, config.default_instance, config.scopes, port).await
    } else {
        run_stdio(clients, config.default_instance, config.scopes).await
    }
}

async fn run_stdio(
    clients: HashMap<String, Arc<ISClient>>,
    default_instance: String,
    scopes: Vec<String>,
) -> anyhow::Result<()> {
    tracing::info!(
        "Starting MCP server (stdio, {} instance(s), default: '{}')",
        clients.len(),
        default_instance,
    );

    let service = WmServer::new(clients, default_instance)
        .with_scopes(scopes)
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

async fn run_http(
    clients: HashMap<String, Arc<ISClient>>,
    default_instance: String,
    scopes: Vec<String>,
    port: u16,
) -> anyhow::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use tokio_util::sync::CancellationToken;

    tracing::info!(
        "Starting MCP server (HTTP on port {}, {} instance(s), default: '{}')",
        port,
        clients.len(),
        default_instance,
    );

    let ct = CancellationToken::new();

    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        json_response: false,
        sse_keep_alive: Some(std::time::Duration::from_secs(30)),
        cancellation_token: ct.child_token(),
        ..Default::default()
    };

    let service: StreamableHttpService<WmServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || {
                Ok(WmServer::new(clients.clone(), default_instance.clone())
                    .with_scopes(scopes.clone()))
            },
            Default::default(),
            config,
        );

    let router = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind port {port}: {e}"))?;

    tracing::info!("MCP HTTP server listening on http://0.0.0.0:{port}/mcp");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
            tracing::info!("Shutting down...");
            ct.cancel();
        })
        .await?;

    Ok(())
}
