//! A minimal file logger (log.txt in the data folder).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;

struct FileLogger {
    file: Mutex<Option<File>>,
    enabled: AtomicBool,
}

static LOGGER: FileLogger = FileLogger { file: Mutex::new(None), enabled: AtomicBool::new(true) };
static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info || (cfg!(debug_assertions) && metadata.target().starts_with("mantice"))
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // ignore the chatter of the GUI dependencies
        let target = record.target();
        if record.level() > log::Level::Error && !target.starts_with("mantice") {
            return;
        }
        let line = format!("{} [{}] {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), record.level(), record.args());
        if cfg!(debug_assertions) {
            eprint!("{line}");
        }
        if self.enabled.load(Ordering::Relaxed)
            && let Some(f) = self.file.lock().as_mut()
        {
            let _ = f.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Some(f) = self.file.lock().as_mut() {
            let _ = f.flush();
        }
    }
}

pub fn init(path: &Path, write_to_file: bool) {
    // keep the log file small: start over when it exceeds 1 MB
    if std::fs::metadata(path).map(|m| m.len() > 1024 * 1024).unwrap_or(false) {
        let _ = std::fs::rename(path, path.with_extension("old.txt"));
    }
    *LOGGER.file.lock() = OpenOptions::new().create(true).append(true).open(path).ok();
    *LOG_PATH.lock() = Some(path.to_path_buf());
    LOGGER.enabled.store(write_to_file, Ordering::Relaxed);
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info });
}

pub fn set_file_logging(enabled: bool) {
    LOGGER.enabled.store(enabled, Ordering::Relaxed);
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.lock().clone()
}
