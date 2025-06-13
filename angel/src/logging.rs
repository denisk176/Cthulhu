use std::fs::File;
use pin_project::pin_project;
use std::io::{Error, Write};
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, ready};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::io::InspectReader;
use tracing::{info, Level};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer, Registry};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use cthulhu_config::angel::AngelConfig;

pub trait SerialIO: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send + Sync> SerialIO for T {}

#[pin_project]
pub struct SerialLogger<IO: SerialIO> {
    #[pin]
    stream: IO,
    buffer: String,
}

impl<IO: SerialIO> SerialLogger<IO> {
    pub fn new(stream: IO,) -> Self {
        Self {
            stream,
            buffer: String::new(),
        }
    }
}

impl<IO: SerialIO> AsyncRead for SerialLogger<IO> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let filled_length = buf.filled().len();
        let me = self.project();
        ready!(me.stream.poll_read(cx, buf))?;
        let data = &buf.filled()[filled_length..];

        me.buffer.extend(String::from_utf8_lossy(data).chars());
        while let Some(pos) = me.buffer.find('\n') {
            let line: String = me.buffer.drain(..=pos).collect();
            let p = line.trim_end();
            let p = strip_ansi_escapes::strip_str(p);
            info!("Serial: {p}");
        }

        Poll::Ready(Ok(()))
    }
}

impl<IO: SerialIO> AsyncWrite for SerialLogger<IO> {
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

#[derive(Clone)]
pub struct TracingTarget {
    target: Arc<Mutex<Option<File>>>,
}

impl TracingTarget {
    pub fn open_file<P: AsRef<Path>>(&self, path: P) -> color_eyre::Result<()> {
        if let Some(p) = path.as_ref().parent() {
            std::fs::create_dir_all(p)?;
        }
        let f = File::create(path)?;
        let mut l = self.target.lock().unwrap();
        *l = Some(f);
        Ok(())
    }
}

pub struct TracingTargetWriter {
    target: TracingTarget,
}

impl Write for TracingTargetWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let t = self.target.target.lock().unwrap();
        if let Some(target) = t.as_ref().as_mut() {
            target.write(buf)
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let t = self.target.target.lock().unwrap();
        if let Some(target) = t.as_ref().as_mut() {
            target.flush()
        } else {
            Ok(())
        }
    }
}

impl<'a> MakeWriter<'a> for TracingTarget {
    type Writer = TracingTargetWriter;

    fn make_writer(&'a self) -> Self::Writer {
        TracingTargetWriter {
            target: self.clone(),
        }
    }
}

pub async fn setup_tracing(config: &AngelConfig) -> color_eyre::Result<TracingTarget> {
    let max_log_level =
        Level::from_str(&(config.log_level.as_ref().unwrap_or(&"info".to_string())))?;
    let target = TracingTarget { target: Arc::new(Mutex::new(None)), };
    let stdsub = tracing_subscriber::fmt::layer()
        .with_filter(LevelFilter::from_level(max_log_level));
    let filesub = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(target.clone())
        .with_filter(LevelFilter::from_level(max_log_level));
    let subscriber = Registry::default().with(stdsub).with(filesub);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(target)
}

pub async fn wrap_raw_serial_log<IO: 'static + AsyncRead + AsyncWrite + Unpin  + Send + Sync>(inp: IO) -> color_eyre::Result<(impl 'static + AsyncRead + AsyncWrite + Unpin  + Send + Sync, TracingTarget)> {
    let target = TracingTarget { target: Arc::new(Mutex::new(None)), };
    let io = {
        let target = target.clone();
        InspectReader::new(inp, move |d| {
            let mut writer = target.make_writer();
            writer.write_all(d).unwrap();
            writer.flush().unwrap();
        })
    };
    Ok((io, target))
}
