use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing::{info};

pub struct TracingWriter {
    cw: ContainedWriter,
    buf: String,
}

impl TracingWriter {
    pub fn new(cw: ContainedWriter) -> Self {
        Self { cw, buf: String::new() }
    }
}

impl Write for TracingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let subscriber = {
            let containedwriter = self.cw.clone();
            tracing_subscriber::fmt()
                .with_writer(move || containedwriter.clone())
                .finish()
        };
        let _guard = tracing::subscriber::set_default(subscriber);

        let l = buf.len();
        let buf2 = String::from_utf8_lossy(buf);
        self.buf.extend(buf2.chars());

        while let Some(pos) = self.buf.find('\n') {
            let line: String = self.buf.drain(..=pos).collect();
            let p = line.trim_end();
            info!("Serial: {p}");
        }

        Ok(l)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct ContainedWriter {
    inner: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
}

impl ContainedWriter {
    pub fn new(writer: Box<dyn Write + Send + 'static>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(writer)),
        }
    }

    pub fn replace(&self, writer: Box<dyn Write + Send + 'static>) {
        let mut inner = self.inner.lock().unwrap();
        *inner = writer;
    }
}

impl Write for ContainedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}