use cthulhu_config::angel::AngelPortConfig;
use tokio::io::{AsyncRead, AsyncWrite};

pub mod rawtcp;
pub mod tty;

pub(crate) trait SwitchSerialPort: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

pub async fn port_from_config(
    c: &AngelPortConfig,
) -> color_eyre::Result<Box<dyn SwitchSerialPort>> {
    match c {
        AngelPortConfig::TTY(config) => Ok(Box::new(
            tty::TTYSwitchSerialPort::new(&config.path, config.baudrate.0).await?,
        )),
        AngelPortConfig::RawTCP(config) => Ok(Box::new(
            rawtcp::RawTCPSwitchSerialPort::new(&config.endpoint).await?,
        )),
    }
}
