use crate::ports::SwitchSerialPort;
use pin_project::pin_project;
use std::io::Error;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use color_eyre::eyre::OptionExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_serial::SerialStream;

#[pin_project]
pub struct TTYSwitchSerialPort {
    #[pin]
    stream: SerialStream,
}

impl TTYSwitchSerialPort {
    pub async fn new<P: AsRef<Path>>(port: P, baudrate: u32) -> color_eyre::Result<Self> {
        let builder = tokio_serial::new(port.as_ref().as_os_str().to_str().ok_or_eyre("failed to convert path")?, baudrate);
        let stream = SerialStream::open(&builder)?;
        Ok(TTYSwitchSerialPort { stream })
    }
}

impl SwitchSerialPort for TTYSwitchSerialPort {}

impl AsyncRead for TTYSwitchSerialPort {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().stream.as_mut().poll_read(cx, buf)
    }
}

impl AsyncWrite for TTYSwitchSerialPort {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.project().stream.as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().stream.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().stream.as_mut().poll_shutdown(cx)
    }
}
