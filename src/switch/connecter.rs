use std::io::{Read, Write};
use std::net::TcpStream;
use color_eyre::eyre;
use tracing::info;
use url::Url;
use crate::config::PortConfig;

pub type SerialConnection = (Box<dyn Read + Send + 'static>, Box<dyn Write + Send + 'static>);

pub fn connect(input: &str) -> color_eyre::Result<SerialConnection> {
    info!("Connecting to serial port {input}...");
    if input.starts_with("/") {
        // It's a file path. We will use serialport-rs.
        let p = serialport::new(input, 9600).open()?;
        let p2 = p.try_clone()?;
        Ok((p, p2))
    } else {
        let u = Url::parse(input)?;
        match u.scheme() {
            "tcp" => {
                let p = TcpStream::connect((u.host_str().ok_or_else(|| eyre::eyre!("no host found in url"))?, u.port().ok_or_else(|| eyre::eyre!("no port found in url"))?))?;
                let p2 = p.try_clone()?;
                Ok((Box::new(p), Box::new(p2)))
            },
            _ => unreachable!(),
        }
    }
}

impl PortConfig {
    pub fn connect(&self) -> color_eyre::Result<SerialConnection> {
        connect(&self.path)
    }
}
