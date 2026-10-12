use anyhow::Result;
use clap::{Parser, Subcommand};
use nixos_control_api_client::Client;

#[derive(Debug, Parser)]
#[command(version, about = "Client for the NixOS control plane daemon")]
struct Args {
    #[arg(
        long,
        env = "NIXOS_CONTROL_URL",
        default_value = "http://127.0.0.1:9834/"
    )]
    url: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Health,
    Capabilities,
    Configuration,
    Options,
    Packages,
    Services,
    System,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new(&args.url)?;
    let value = match args.command {
        Command::Health => serde_json::to_value(client.health().await?)?,
        Command::Capabilities => serde_json::to_value(client.capabilities().await?)?,
        Command::Configuration => serde_json::to_value(client.configuration().await?)?,
        Command::Options => serde_json::to_value(client.options().await?)?,
        Command::Packages => serde_json::to_value(client.packages().await?)?,
        Command::Services => serde_json::to_value(client.services().await?)?,
        Command::System => serde_json::to_value(client.system_summary().await?)?,
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
