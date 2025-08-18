// src/logger.rs
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};

use chrono::Local;
use directories::BaseDirs;

const MAX_LOG_ROTATIONS: usize = 3;
const DEFAULT_FLUSH_INTERVAL_MS: u64 = 300;

/// Log level for formatting and filtering (simple and cheap).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiny message passed to the logging worker.
struct LogMsg {
    level: Level,
    line: String,
}

/// Public trait used by the rest of your code.
pub trait ActionLog: Send + Sync {
    fn log(&self, message: &str);
    fn log_level(&self, level: Level, message: &str);
}

pub trait ActionLogExt: ActionLog {
    fn log_msg<M: AsRef<str>>(&self, message: M) {
        self.log(message.as_ref());
    }
    fn log_level_msg<M: AsRef<str>>(&self, level: Level, message: M) {
        self.log_level(level, message.as_ref());
    }
}
impl<T: ActionLog + ?Sized> ActionLogExt for T {}

/// Backing implementation: file + background thread + rotation.
pub struct FileLogger {
    tx: mpsc::Sender<LogMsg>,
    // Keep a handle so we can request a flush/close on drop if needed later.
    _worker: thread::JoinHandle<()>,
}

#[derive(Clone)]
pub struct LoggerConfig {
    /// Maximum bytes before rotating to `.1.log`. None = no size rotation.
    pub max_bytes: Option<u64>,
    /// Number of rotated files to keep (e.g., plugin.1.log .. plugin.N.log)
    pub max_rotations: usize,
    /// Periodic flush interval for the background writer.
    pub flush_interval: Duration,
    /// Minimum level to write (others dropped before formatting).
    pub min_level: Level,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            max_bytes: Some(2 * 1024 * 1024), // 2 MiB
            max_rotations: MAX_LOG_ROTATIONS,
            flush_interval: Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS),
            min_level: Level::Debug,
        }
    }
}

#[macro_export]
macro_rules! debug {
    (
        $logger:expr,
        $($arg:tt)*
    ) => {
        {
        use $crate::ActionLogExt as _;
        $logger.log_level_msg($crate::Level::Debug, format!($($arg)*));
        }
    };
}
#[macro_export]
macro_rules! info {
    (
        $logger:expr,
        $($arg:tt)*
    ) => {
        {
        use $crate::ActionLogExt as _;
        $logger.log_level_msg($crate::Level::Info, format!($($arg)*));
        }
    };
}
#[macro_export]
macro_rules! warn {
    (
        $logger:expr,
        $($arg:tt)*
    ) => {
        {
        use $crate::ActionLogExt as _;
        $logger.log_level_msg($crate::Level::Warn, format!($($arg)*));
        }
    };
}
#[macro_export]
macro_rules! error {
    (
        $logger:expr,
        $($arg:tt)*
    ) => {
        {
        use $crate::ActionLogExt as _;
        $logger.log_level_msg($crate::Level::Error, format!($($arg)*));
        }
    };
}

impl FileLogger {
    /// Initialize a logger at an explicit path (rotates on startup).
    pub fn init_with_config<P: AsRef<Path>>(path: P, cfg: LoggerConfig) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        rotate_logs_startup(&path, cfg.max_rotations)?;

        // Open (append) the active file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open log file: {e}"))?;

        let (tx, rx) = mpsc::channel::<LogMsg>();

        // Clone path for the worker thread so we can keep the original
        let worker_path = path.clone();

        // Writer state kept inside worker thread
        let worker = thread::spawn(move || {
            let mut file = file;
            let mut bytes_written: u64 = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
            let mut last_flush = std::time::Instant::now();

            loop {
                // Use recv_timeout so we can periodically flush.
                match rx.recv_timeout(cfg.flush_interval) {
                    Ok(msg) => {
                        if msg.level < cfg.min_level {
                            continue;
                        }
                        // Format line with timestamp + level once here.
                        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                        let formatted =
                            format!("[{}] {:<5} {}\n", timestamp, msg.level.as_str(), msg.line);

                        // Rotate by size if configured
                        if let Some(max) = cfg.max_bytes {
                            if bytes_written + (formatted.len() as u64) > max {
                                // Close current file by dropping it, rotate files,
                                // and reopen a fresh one at the same path.
                                if let Err(e) = rotate_logs_startup(&worker_path, cfg.max_rotations)
                                {
                                    // We can't log about logging, but try stderr.
                                    let _ = writeln!(io::stderr(), "log rotation error: {e}");
                                }
                                match OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(&worker_path)
                                {
                                    Ok(f) => {
                                        file = f;
                                        bytes_written = 0;
                                    }
                                    Err(e) => {
                                        let _ = writeln!(io::stderr(), "log reopen error: {e}");
                                        // Keep trying to write to old file to avoid losing logs
                                    }
                                }
                            }
                        }

                        if let Err(e) = file.write_all(formatted.as_bytes()) {
                            let _ = writeln!(io::stderr(), "Failed to write to log file: {e}");
                        } else {
                            bytes_written += formatted.len() as u64;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // periodic flush
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }

                // Flush opportunistically
                if last_flush.elapsed() >= cfg.flush_interval {
                    let _ = file.flush();
                    last_flush = std::time::Instant::now();
                }
            }

            // Final flush on exit
            let _ = file.flush();
        });

        Ok(Self {
            tx,
            _worker: worker,
        })
    }

    /// Initialize like before, with default config.
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        Self::init_with_config(path, LoggerConfig::default())
    }

    /// Previous convenience: resolve `%APPDATA%/…/PLUGIN_UUID/plugin.log`.
    pub fn from_appdata(plugin_id: &str) -> Result<Self, String> {
        let base = BaseDirs::new().ok_or("Could not find user data directory")?;
        let log_dir = base.data_dir().join(plugin_id);
        fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log directory: {e}"))?;
        let log_file = log_dir.join("plugin.log");
        Self::init(log_file)
    }
}

impl Drop for FileLogger {
    fn drop(&mut self) {
        // Drop the sender to signal shutdown
        // (tx is dropped automatically; being explicit is clear)
        // Then join the worker so we know the final flush happened.
        // If the worker is blocked, the recv_timeout upper-bounds the wait.
        if let Some(worker) = std::mem::replace(&mut self._worker, thread::spawn(|| {}))
            .join()
            .err()
        {
            // Best-effort: you could write to stderr here if you want.
            let _ = writeln!(io::stderr(), "logger worker join failed: {worker:?}");
        }
    }
}

impl ActionLog for FileLogger {
    fn log(&self, message: &str) {
        let _ = self.tx.send(LogMsg {
            level: Level::Info,
            line: message.to_string(),
        });
    }
    fn log_level(&self, level: Level, message: &str) {
        let _ = self.tx.send(LogMsg {
            level,
            line: message.to_string(),
        });
    }
}

// ---- helpers ----

fn rotate_logs_startup(base_path: &Path, max_rotations: usize) -> Result<(), String> {
    // Move plugin.(N-1).log -> plugin.N.log, then plugin.log -> plugin.1.log
    for i in (1..=max_rotations).rev() {
        let src = base_path.with_extension(format!("{i}.log"));
        let dst = base_path.with_extension(format!("{}.log", i + 1));
        if src.exists() {
            if i == max_rotations {
                fs::remove_file(&src).map_err(|e| format!("Failed to remove old log: {e}"))?;
            } else {
                fs::rename(&src, &dst).map_err(|e| format!("Failed to rotate log: {e}"))?;
            }
        }
    }
    if max_rotations > 0 && base_path.exists() {
        let rotated = base_path.with_extension("1.log");
        fs::rename(base_path, rotated).map_err(|e| format!("Failed to archive log: {e}"))?;
    }
    Ok(())
}
