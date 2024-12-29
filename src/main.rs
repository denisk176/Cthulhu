#![feature(trait_upcasting)]

use clap::Parser;
use tracing::info;
use crate::args::{Cli, CliMode, CliMulti};
use crate::switch::{run_port_sync, spawn_port};

mod switch;
pub mod args;
pub mod config;
pub mod web;

fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();

    match &cli.mode {
        CliMode::Single(config) => {
            tracing_subscriber::fmt::init();
            run_port_sync(config)?;
        }
        CliMode::Multi(mcli) => {
            multi_port_mode(&cli, mcli)?;
        }
    }

    Ok(())
}

fn multi_port_mode(_cli: &Cli, mcli: &CliMulti) -> color_eyre::Result<()> {
    let config = mcli.read_config()?;
    // We configure one per thread.
    let subscriber = tracing_subscriber::fmt().finish();
    let _guard = tracing::subscriber::set_default(subscriber);


    info!("Spawning {} ports...", config.ports.len());
    let ports = config.ports.iter().map(|c| spawn_port(c.clone())).collect::<color_eyre::Result<Vec<_>>>()?;

    info!("Spawning tokio runtime...");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    info!("Switching to async code...");
    runtime.block_on(async move {
        web::web_main(config, ports).await
    })?;

    Ok(())
}
