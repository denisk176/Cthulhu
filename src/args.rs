use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use tracing::info;
use crate::config::Config;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub mode: CliMode,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliMode {
    Single(crate::config::PortConfig),
    Multi(CliMulti),
}

#[derive(Debug, Clone, Args)]
pub struct CliMulti {
    #[clap(long, short)]
    config: PathBuf,
}

impl CliMulti {
    pub fn read_config(&self) -> color_eyre::Result<Config> {
        info!("Reading config file {:?}...", &self.config);
        let f = std::fs::read_to_string(&self.config)?;
        let c = toml::from_str(&f)?;
        Ok(c)
    }
}