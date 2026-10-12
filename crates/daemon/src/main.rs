use anyhow::Context;
use clap::Parser;
use nixos_control_daemon::{DaemonConfig, router};
use nixos_control_nix::ConfigurationTarget;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(version, about = "Headless NixOS control plane daemon")]
struct Args {
    #[arg(long, env = "NIXOS_CONTROL_LISTEN", default_value = "127.0.0.1:9834")]
    listen: SocketAddr,
    #[arg(long, env = "NIXOS_CONTROL_COMMAND_TIMEOUT", default_value_t = 30)]
    command_timeout_seconds: u64,
    #[arg(
        long,
        env = "NIXOS_CONTROL_CONFIGURATION_PATH",
        default_value = "/etc/nixos",
        conflicts_with = "flake_uri"
    )]
    configuration_path: PathBuf,
    #[arg(
        long,
        env = "NIXOS_CONTROL_FLAKE_URI",
        conflicts_with = "configuration_path"
    )]
    flake_uri: Option<String>,
    #[arg(long, env = "NIXOS_CONTROL_HOST")]
    host: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let args = Args::parse();
    let listener = TcpListener::bind(args.listen)
        .await
        .with_context(|| format!("failed to bind {}", args.listen))?;
    info!(listen = %args.listen, "daemon listening");
    axum::serve(
        listener,
        router(DaemonConfig {
            command_timeout: Duration::from_secs(args.command_timeout_seconds),
            configuration_target: ConfigurationTarget {
                path: args.flake_uri.is_none().then_some(args.configuration_path),
                flake_uri: args.flake_uri,
                host: args.host,
            },
        }),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("daemon failed")
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.ok() };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .ok()?
            .recv()
            .await
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<Option<()>>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
