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