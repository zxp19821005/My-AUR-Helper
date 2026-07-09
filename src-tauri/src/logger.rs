use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex as StdMutex;

/// 日志轮转配置
#[derive(Debug, Clone, Copy)]
pub struct LogSettings {
    /// 单个日志文件大小上限（字节），默认 10MB
    pub max_size: u64,
    /// 保留的日志文件最大数量，默认 7
    pub max_files: usize,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024,
            max_files: 7,
        }
    }
}

static LOG_SETTINGS: StdMutex<LogSettings> = StdMutex::new(LogSettings {
    max_size: 10 * 1024 * 1024,
    max_files: 7,
});

/// 更新日志轮转配置（运行时调用）
pub fn update_log_settings(max_size: u64, max_files: usize) {
    if let Ok(mut settings) = LOG_SETTINGS.lock() {
        settings.max_size = max_size;
        settings.max_files = max_files;
    }
}

/// 获取当前日志配置
pub fn get_log_settings() -> LogSettings {
    LOG_SETTINGS.lock().map(|s| *s).unwrap_or_default()
}

/// 带日志轮转的文件日志记录器
pub struct RotatingLogger {
    log_dir: PathBuf,
    prefix: String,
    state: StdMutex<LoggerState>,
}

struct LoggerState {
    current_date: String,
    file: Option<BufWriter<File>>,
    file_size: u64,
}

impl RotatingLogger {
    /// 创建日志记录器
    pub fn new(log_dir: PathBuf, prefix: String) -> Self {
        fs::create_dir_all(&log_dir).ok();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let (file, file_size) = Self::open_file(&log_dir, &prefix, &today);
        Self {
            log_dir,
            prefix,
            state: StdMutex::new(LoggerState {
                current_date: today,
                file,
                file_size,
            }),
        }
    }

    fn open_file(log_dir: &Path, prefix: &str, date: &str) -> (Option<BufWriter<File>>, u64) {
        let path = log_dir.join(format!("{}-{}.log", prefix, date));
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => {
                let size = file.metadata().map(|m| m.len()).unwrap_or(0);
                (Some(BufWriter::new(file)), size)
            }
            Err(_) => (None, 0),
        }
    }

    fn rotate_file(log_dir: &Path, prefix: &str, date: &str) {
        let current_path = log_dir.join(format!("{}-{}.log", prefix, date));
        let mut counter = 0u32;
        loop {
            let rotated = log_dir.join(format!("{}-{}.{}.log", prefix, date, counter));
            if !rotated.exists() {
                let _ = fs::rename(&current_path, &rotated);
                break;
            }
            counter += 1;
        }
    }

    fn cleanup(log_dir: &Path, prefix: &str, max_files: usize) {
        let Ok(entries) = fs::read_dir(log_dir) else { return };
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&format!("{}-", prefix)) && n.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();
        if files.len() <= max_files {
            return;
        }
        files.sort_by_key(|e| {
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        for old in files.iter().rev().skip(max_files) {
            let _ = fs::remove_file(old.path());
        }
    }

    /// 初始化全局日志记录器
    pub fn init(self) -> Result<(), SetLoggerError> {
        let logger = Box::new(self);
        let logger = Box::leak(logger);
        log::set_logger(logger).map(|()| log::set_max_level(LevelFilter::Debug))
    }
}

impl Log for RotatingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let level = match record.level() {
            Level::Error => "错误",
            Level::Warn => "警告",
            Level::Info => "信息",
            Level::Debug => "调试",
            Level::Trace => "跟踪",
        };
        let msg = format!(
            "{} - {}: [{}] {}",
            now.format("%Y-%m-%d %H:%M:%S%.3f"),
            level,
            record.target(),
            record.args()
        );

        println!("{}", msg);

        let settings = get_log_settings();
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        if date != state.current_date {
            state.file = None;
            let (file, size) = Self::open_file(&self.log_dir, &self.prefix, &date);
            state.file = file;
            state.file_size = size;
            state.current_date = date.clone();
            Self::cleanup(&self.log_dir, &self.prefix, settings.max_files);
        }

        if state.file_size > settings.max_size {
            state.file = None;
            Self::rotate_file(&self.log_dir, &self.prefix, &state.current_date);
            let (file, size) = Self::open_file(&self.log_dir, &self.prefix, &state.current_date);
            state.file = file;
            state.file_size = size;
        }

        if let Some(ref mut file) = state.file {
            let _ = writeln!(file, "{}", msg);
            let _ = file.flush();
            state.file_size += msg.len() as u64 + 1;
        }
    }

    fn flush(&self) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(ref mut file) = state.file {
                let _ = file.flush();
            }
        }
    }
}
