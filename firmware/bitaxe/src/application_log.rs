//! Bounded Rust diagnostic routing; ESP-IDF C console output stays on UART0.

use log::{Log, Metadata, Record};
use std::fmt::Write;

static LOGGER: ApplicationLogger = ApplicationLogger;
struct ApplicationLogger;

pub(crate) fn initialize() -> anyhow::Result<()> {
    log::set_logger(&LOGGER).map_err(|_| anyhow::anyhow!("application_logger_already_owned"))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

impl Log for ApplicationLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut line = BoundedLine {
            bytes: [0; 1024],
            used: 0,
        };
        if write!(&mut line, "{}", record.args()).is_ok() {
            if let Ok(text) = std::str::from_utf8(&line.bytes[..line.used]) {
                crate::bwg_worker_usb::diagnostic(text);
            }
        }
    }
    fn flush(&self) {}
}

struct BoundedLine {
    bytes: [u8; 1024],
    used: usize,
}
impl Write for BoundedLine {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let end = self.used.checked_add(text.len()).ok_or(std::fmt::Error)?;
        if end > self.bytes.len() {
            return Err(std::fmt::Error);
        }
        self.bytes[self.used..end].copy_from_slice(text.as_bytes());
        self.used = end;
        Ok(())
    }
}
