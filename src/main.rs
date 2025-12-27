mod config;
mod crypto;
mod encoding;
mod error;
mod proxy;
mod server;

use clap::Parser;
use config::{Cli, Command};
use crypto::generate_digest;
use encoding::{encode_url_base64, encode_url_hex};
use proxy::ProxyClient;
use server::{create_router, AppState};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Sign { url, base, base64 }) => {
            let key = cli
                .key
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("CAMO_KEY is required for signing"))?;

            let digest = generate_digest(key, url);
            let encoded_url = if *base64 {
                encode_url_base64(url)
            } else {
                encode_url_hex(url)
            };

            if base.is_empty() {
                println!("Digest: {}", digest);
                println!("Encoded URL: {}", encoded_url);
                println!("Path: /{}/{}", digest, encoded_url);
            } else {
                let base = base.trim_end_matches('/');
                println!("{}/{}/{}", base, digest, encoded_url);
            }
        }
        Some(Command::Serve) | None => {
            let key = cli
                .key
                .clone()
                .ok_or_else(|| anyhow::anyhow!("CAMO_KEY is required"))?;

            // Initialize logging
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
                )
                .init();

            // Initialize metrics if enabled
            if cli.metrics {
                let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
                builder
                    .install()
                    .expect("Failed to install Prometheus recorder");
            }

            let listen = cli.listen.clone();
            let config = Arc::new(Cli {
                key: Some(key),
                ..cli
            });

            // Create proxy client
            let proxy = ProxyClient::new(config.clone());

            // Create app state
            let state = Arc::new(AppState { config, proxy });

            // Create router
            let app = create_router(state);

            // Start server
            let listener = tokio::net::TcpListener::bind(&listen).await?;
            info!("camo-rs listening on {}", listen);

            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
