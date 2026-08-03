/**
 * log_reader.rs - 日志文件读取模块
 *
 * 功能：
 * - 从日志文件读取并解析日志条目
 * - 支持查找最新日志文件（优先当天）
 * - 支持清空当天日志文件
 * - 返回倒序排列的日志条目（最新的在前）
 */
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::errors::AppResult;
use log::debug;

/// 展开路径中的 ~ 为实际的用户主目录
fn expand_tilde(path: &str) -> String {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]).to_string_lossy().to_string();
        }
    }
    path.to_string()
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
    let log_dir_expanded = expand_tilde(log_dir);
    let log_path = Path::new(&log_dir_expanded);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    debug!(
        "读取日志: log_dir={}, prefix={}, today={}",
        log_dir_expanded, prefix, today
    );

    // 优先查找当天日志，如果不存在则查找最新日志文件
    let target_file = find_latest_log_file(log_path, prefix, &today)?;
    if target_file.is_none() {
        debug!("未找到日志文件");
        return Ok(Vec::new());
    }
    let file_path = target_file.unwrap();
    debug!("找到日志文件: {:?}", file_path);

    let file = File::open(&file_path)
        .map_err(|e| crate::errors::AppError::FileOperation(format!("打开日志文件失败: {}", e)))?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    let mut parsed_count = 0;
    let mut total_lines = 0;
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        total_lines += 1;
        if let Some(entry) = parse_log_line(&line) {
            entries.push(entry);
            parsed_count += 1;
        }
    }

    debug!(
        "日志解析: 总行数={}, 解析成功={}",
        total_lines, parsed_count
    );

    // 限制返回数量（保留最新的 limit 条）
    if entries.len() > limit {
        let start = entries.len() - limit;
        entries = entries.split_off(start);
    }

    debug!("返回日志条目数: {}", entries.len());
    Ok(entries)
}

/// 从指定位置读取新增日志（类似 tail -f）
/// @param log_dir - 日志目录路径
/// @param prefix - 日志文件前缀
/// @param last_position - 上次读取的文件位置（字节偏移）
/// @param last_file - 上次读取的文件名
/// @param limit - 返回的最大条数
/// @returns (日志条目列表, 新的文件位置, 当前文件名)
pub fn read_new_log_entries(
    log_dir: &str,
    prefix: &str,
    last_position: u64,
    last_file: &str,
    limit: usize,
) -> AppResult<(Vec<FileLogEntry>, u64, String)> {
    let log_dir_expanded = expand_tilde(log_dir);
    let log_path = Path::new(&log_dir_expanded);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 查找当前应该读取的文件
    let target_file = find_latest_log_file(log_path, prefix, &today)?;
    if target_file.is_none() {
        return Ok((Vec::new(), 0, String::new()));
    }
    let file_path = target_file.unwrap();
    let current_file = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // 如果文件切换了，从头读取
    let position = if current_file != last_file {
        0
    } else {
        last_position
    };

    let mut file = File::open(&file_path)
        .map_err(|e| crate::errors::AppError::FileOperation(format!("打开日志文件失败: {}", e)))?;

    // 获取文件大小
    let metadata = file.metadata().map_err(|e| {
        crate::errors::AppError::FileOperation(format!("获取文件元数据失败: {}", e))
    })?;
    let file_size = metadata.len();

    // 如果文件被截断（清空），从头读取
    let actual_position = if position > file_size { 0 } else { position };

    // 如果位置没有变化且不是从头读取，说明没有新内容，直接返回
    if actual_position == position && position == file_size {
        return Ok((Vec::new(), position, current_file));
    }

    // 定位到上次读取的位置
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(actual_position))
        .map_err(|e| crate::errors::AppError::FileOperation(format!("定位文件位置失败: {}", e)))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut new_position = actual_position;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        new_position += line.len() as u64 + 1; // +1 for newline
        if let Some(entry) = parse_log_line(&line) {
            entries.push(entry);
        }
        if entries.len() >= limit {
            break;
        }
    }

    Ok((entries, new_position, current_file))
}

/// 清空当天日志文件内容
pub fn clear_today_log(log_dir: &str, prefix: &str) -> AppResult<()> {
    let log_dir_expanded = expand_tilde(log_dir);
    let log_path = Path::new(&log_dir_expanded);
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
fn find_latest_log_file(log_dir: &Path, prefix: &str, today: &str) -> AppResult<Option<PathBuf>> {
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
    // 尝试匹配格式: "2026-07-31 12:34:56.789 - INFO: [module] message"
    let parts: Vec<&str> = line.splitn(2, " - ").collect();
    if parts.len() != 2 {
        return None;
    }
    let timestamp = parts[0].trim();
    let rest = parts[1];

    // 使用 ": [" 作为分隔符，避免消息内容中的 ": " 干扰解析
    let separator = rest.find(": [")?;
    let level = rest[..separator].trim();

    let after_separator = &rest[separator + 3..]; // 跳过 ": ["

    let (module, message) = if let Some(end) = after_separator.find("] ") {
        let module = &after_separator[..end];
        let message = &after_separator[end + 2..];
        (module.to_string(), message.to_string())
    } else {
        (after_separator.to_string(), String::new())
    };

    Some(FileLogEntry {
        timestamp: timestamp.to_string(),
        level: level.to_string(),
        module,
        message,
    })
}
