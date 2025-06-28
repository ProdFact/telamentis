//! Command-line interface for TelaMentis

use clap::{Parser, Subcommand};
use std::process;
use tracing::{error, info, Level};
use tracing_subscriber;

mod cli;
mod commands;
mod config;
mod output;
mod client;

use cli::*;
use config::KgctlConfig;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    // Initialize logging
    let log_level = match args.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Load configuration
    let config = match KgctlConfig::load(&args.config).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Override config with CLI args
    let config = config.with_overrides(&args);

    info!("Starting kgctl with endpoint: {}", config.endpoint);

    // Execute command
    let result = match args.command {
        Commands::Tenant { command } => {
            commands::tenant::handle_tenant_command(command, &config).await
        }
        Commands::Ingest { command } => {
            commands::ingest::handle_ingest_command(command, &config).await
        }
        Commands::Export { command } => {
            commands::export::handle_export_command(command, &config).await
        }
        Commands::Query { command } => {
            commands::query::handle_query_command(command, &config).await
        }
        Commands::Health => {
            commands::health::handle_health_command(&config).await
        }
    };

    match result {
        Ok(_) => {
            info!("Command completed successfully");
        }
        Err(e) => {
            error!("Command failed: {}", e);
            process::exit(1);
        }
    }
}