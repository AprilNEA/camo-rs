use camo::{
    server::{
        config::{Command, Config},
        router::{create_router, AppState},
    },
    {CamoUrl, Encoding},
};
use clap::Parser;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Config::parse();

    let key = cli
        .key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("CAMO_KEY is required for signing"))?;

    match &cli.command {
        Some(Command::Sign { url, base, base64 }) => {
            let camo = CamoUrl::new(key).with_encoding(if *base64 {
                Encoding::Base64
            } else {
                Encoding::Hex
            });

            let signed = camo.sign(url);

            if base.is_empty() {
                println!("Digest: {}", signed.digest);
                println!("Encoded URL: {}", signed.encoded_url);
                println!("Path: {}", signed.to_path());
            } else {
                println!("{}", signed.to_url(base));
            }
        }
        Some(Command::Serve) | None => {
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
            let config = Arc::new(Config {
                key: Some(key.clone()),
                ..cli
            });

            // Create app state
            let state = Arc::new(AppState::from_config(&config));

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
