use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex as StdMutex;

use crate::errors::AppResult;

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
        let Ok(entries) = fs::read_dir(log_dir) else {
            return;
        };
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

/// 日志条目（从文件解析）
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}

/// 读取日志文件，返回倒序排列的最新日志条目
/// @param log_dir - 日志目录路径
/// @param prefix - 日志文件前缀
/// @param limit - 返回的最大条数
/// @returns 日志条目列表（按时间倒序，最新的在前）
pub fn read_log_entries(log_dir: &str, prefix: &str, limit: usize) -> AppResult<Vec<FileLogEntry>> {
    let log_path = Path::new(log_dir);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 优先查找当天日志，如果不存在则查找最新日志文件
    let target_file = find_latest_log_file(log_path, prefix, &today)?;
    if target_file.is_none() {
        return Ok(Vec::new());
    }
    let file_path = target_file.unwrap();

    let file = File::open(&file_path).map_err(|e| {
        crate::errors::AppError::FileOperation(format!("打开日志文件失败: {}", e))
    })?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Some(entry) = parse_log_line(&line) {
            entries.push(entry);
        }
    }

    // 倒序排列（最新的在前）
    entries.reverse();

    // 限制返回数量
    if entries.len() > limit {
        entries.truncate(limit);
    }

    Ok(entries)
}

/// 清空当天日志文件内容
pub fn clear_today_log(log_dir: &str, prefix: &str) -> AppResult<()> {
    let log_path = Path::new(log_dir);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let file_path = log_path.join(format!("{}-{}.log", prefix, today));

    if file_path.exists() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| {
                crate::errors::AppError::FileOperation(format!("清空日志文件失败: {}", e))
            })?;
    }

    Ok(())
}

/// 查找最新的日志文件（优先当天，否则找最近修改的）
fn find_latest_log_file(
    log_dir: &Path,
    prefix: &str,
    today: &str,
) -> AppResult<Option<PathBuf>> {
    // 先尝试当天文件
    let today_file = log_dir.join(format!("{}-{}.log", prefix, today));
    if today_file.exists() {
        return Ok(Some(today_file));
    }

    // 查找所有匹配的日志文件
    let Ok(entries) = fs::read_dir(log_dir) else {
        return Ok(None);
    };

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

    if files.is_empty() {
        return Ok(None);
    }

    // 按修改时间排序，返回最新的
    files.sort_by_key(|e| {
        e.metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    Ok(files.last().map(|e| e.path()))
}

/// 解析单行日志
/// 格式: YYYY-MM-DD HH:MM:SS.mmm - 级别: [模块] 消息
fn parse_log_line(line: &str) -> Option<FileLogEntry> {
    // 尝试匹配格式: "2026-07-31 12:34:56.789 - 信息: [module] message"
    let parts: Vec<&str> = line.splitn(2, " - ").collect();
    if parts.len() != 2 {
        return None;
    }
    let timestamp = parts[0].trim();
    let rest = parts[1];

    // 解析 "级别: [模块] 消息"
    let colon_pos = rest.find(": ")?;
    let level = rest[..colon_pos].trim();

    let after_colon = &rest[colon_pos + 2..];
    let (module, message) = if after_colon.starts_with('[') {
        if let Some(end) = after_colon.find("] ") {
            let module = &after_colon[1..end];
            let message = &after_colon[end + 2..];
            (module.to_string(), message.to_string())
        } else {
            (String::new(), after_colon.to_string())
        }
    } else {
        (String::new(), after_colon.to_string())
    };

    Some(FileLogEntry {
        timestamp: timestamp.to_string(),
        level: level.to_string(),
        module,
        message,
    })
}