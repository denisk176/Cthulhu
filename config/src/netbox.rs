use std::path::Path;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxConfig {
    #[serde(rename = "NetBox")]
    pub netbox: NetboxNBConfig,

    #[serde(rename = "Heaven")]
    pub heaven: NetboxHeavenConfig,
}
impl NetboxConfig {
    pub async fn from_file<P: AsRef<Path>>(p: P) -> color_eyre::Result<Self> {
        info!("Using config file: {}", p.as_ref().display());
        let d = tokio::fs::read_to_string(p).await?;
        let d = toml::from_str(d.as_str())?;
        Ok(d)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxHeavenConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxNBConfig {
    pub token: String,
    pub url: String,
    pub target_status: String,
}
