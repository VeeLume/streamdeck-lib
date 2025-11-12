use chrono::TimeZone;
// src/telemetry.rs
use directories::BaseDirs;
use std::{fs, io, path::PathBuf};
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_KEEP_RUNS: usize = 4;

fn logs_dir(plugin_id: &str) -> io::Result<PathBuf> {
    let base = BaseDirs::new().ok_or_else(|| io::Error::other("no home dir"))?;
    let dir = base.data_dir().join(plugin_id);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn run_log_path(dir: &PathBuf, prefix: &str) -> PathBuf {
    // Use local time; keep it simple and avoid extra deps for the filename.
    // YYYYMMDD-HHMMSS-PID
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let stamp = chrono::Local
        .timestamp_opt(now as i64, 0)
        .single()
        .map(|dt| dt.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| now.to_string());
    let pid = std::process::id();
    dir.join(format!("{prefix}-{stamp}-{pid}.log"))
}

fn cleanup_old_runs(dir: &PathBuf, prefix: &str, keep: usize) {
    // Delete oldest files matching "<prefix>-*.log", keep newest `keep`
    let mut entries: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    if let Ok(read) = fs::read_dir(dir) {
        for e in read.flatten() {
            let p = e.path();
            if let (Some(name), true) = (
                p.file_name().and_then(|s| s.to_str()),
                p.extension().map(|e| e == "log").unwrap_or(false),
            ) {
                if name.starts_with(prefix) {
                    let mtime = e
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    entries.push((mtime, p));
                }
            }
        }
    }
    entries.sort_by_key(|(t, _)| *t);
    let to_delete = entries.len().saturating_sub(keep);
    for (_, p) in entries.into_iter().take(to_delete) {
        let _ = fs::remove_file(p);
    }
}

/// Initialize tracing for this process:
/// - One file per run
/// - Keep the newest `keep_runs` files (delete older)
/// - Respects RUST_LOG (defaults to "info")
///
/// Return value must be kept alive to flush logs on exit.
pub fn init(plugin_id: &str) -> WorkerGuard {
    init_with(plugin_id, "plugin", DEFAULT_KEEP_RUNS)
}

/// Same as `init` but lets you set the file prefix and how many runs to keep.
pub fn init_with(plugin_id: &str, file_prefix: &str, keep_runs: usize) -> WorkerGuard {
    let dir = logs_dir(plugin_id).expect("failed to create logs dir");
    cleanup_old_runs(&dir, file_prefix, keep_runs);

    let file = run_log_path(&dir, file_prefix);
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .unwrap_or_else(|e| panic!("failed to open log file {file:?}: {e}"));

    let (nb_writer, guard) = non_blocking(file);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Timestamp like "2025-11-12 14:03:31"
    struct ChronoLocalTime;
    impl FormatTime for ChronoLocalTime {
        fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
            let now = chrono::Local::now();
            write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S"))
        }
    }

    let timer = ChronoLocalTime;

    let fmt_layer = fmt::layer()
        .with_writer(nb_writer)
        .with_timer(timer)
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    // Try to install a global subscriber; if one already exists, do nothing.
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init();

    guard
}
