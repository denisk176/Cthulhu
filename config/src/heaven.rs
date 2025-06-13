use serde::Deserialize;
use std::path::Path;
use tracing::info;

#[derive(Deserialize, Debug, Clone)]
pub struct HeavenConfig {
    pub log_level: Option<String>,

    #[serde(rename = "Web")]
    pub web: HeavenWebConfig,
    #[serde(rename = "MQTT")]
    pub mqtt: HeavenMQTTConfig,
}

impl HeavenConfig {
    pub async fn from_file<P: AsRef<Path>>(p: P) -> color_eyre::Result<Self> {
        info!("Using config file: {}", p.as_ref().display());
        let d = tokio::fs::read_to_string(p).await?;
        let d = toml::from_str(d.as_str())?;
        Ok(d)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HeavenWebConfig {
    pub listen_address: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct HeavenMQTTConfig {
    pub id: Option<String>,
    pub host: String,
    pub port: u16,
}
