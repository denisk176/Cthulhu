use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use crate::ports::SwitchSerialPort;
use pin_project::pin_project;


#[pin_project]
pub struct RawTCPSwitchSerialPort {
    #[pin]
    stream: TcpStream,
}

impl RawTCPSwitchSerialPort {
    pub async fn new(conn: &str) -> color_eyre::Result<Self> {
        let stream = TcpStream::connect(conn).await?;
        Ok(RawTCPSwitchSerialPort { stream, })
    }
}

impl SwitchSerialPort for RawTCPSwitchSerialPort {

}

impl AsyncRead for RawTCPSwitchSerialPort {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        self.project().stream.as_mut().poll_read(cx, buf)
    }
}

impl AsyncWrite for RawTCPSwitchSerialPort {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        self.project().stream.as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().stream.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.project().stream.as_mut().poll_shutdown(cx)
    }
}
