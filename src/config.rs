use clap::Args;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub listen_address: String,
    pub ports: Vec<PortConfig>,
}

#[derive(Clone, Debug, Args, Deserialize)]
pub struct PortConfig {
    pub path: String,
    pub label: String,
}
