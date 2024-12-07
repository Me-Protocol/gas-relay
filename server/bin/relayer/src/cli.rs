use clap::Parser;
use primitives::configs::RelayerConfig;
use tasks::{monitor::MonitorTask, relay::ServerTask, spawn_tasks};
use toml::from_str;
use tracing_subscriber::{filter::LevelFilter, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliConfig {
    #[arg(short, long)]
    pub config_path: String,
}

/// Main entry point for the CLI
///
/// Parses the CLI arguments and runs the appropriate subcommand.
/// Listens for a ctrl-c signal and shuts down all components when received.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    logger_setup()?;
    let cli_config = CliConfig::parse();
    let config_as_string = std::fs::read_to_string(cli_config.config_path)?;
    let config: RelayerConfig = from_str(&config_as_string)?;

    // server config
    let server_config = config.clone().server;

    tracing::info!("Starting Relay with config: {:?}", config.clone());

    spawn_tasks(
        ServerTask::new(server_config).boxed(),
        MonitorTask::new("".to_string()).boxed(),
        tokio::signal::ctrl_c(),
    )
    .await;

    Ok(())
}

// this function is for setting up the logging process
pub fn logger_setup() -> Result<(), anyhow::Error> {
    let filter =
        tracing_subscriber::EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    Ok(())
}
