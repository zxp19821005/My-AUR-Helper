/**
 * parse.rs - AUR 元数据归档解析（gzip 解压、字段裁剪、索引构建）
 *
 * 功能：把 .gz 归档字节流解析为 pkgname -> 精简包信息 的映射；
 *       仅保留 parse_aur_fields 所需字段以控制缓存内存；
 *       识别 Anubis 反爬返回的 HTML 并给出可定位错误，避免静默降级。
 */
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::errors::{AppError, AppResult};

/// 保留进索引的字段，与 software_sync::utils::parse_aur_fields 对齐
const KEPT_FIELDS: &[&str] = &[
    "Name",
    "Version",
    "Description",
    "URL",
    "LastModified",
    "License",
    "Depends",
    "MakeDepends",
    "OptDepends",
    "OutOfDate",
    "PackageBase",
    "Maintainer",
];

/// 取字符串前 n 个字符，用于日志预览（按字符截断，不破坏 UTF-8 边界）
fn truncate(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// 调用系统 gzip 解压字节流（同步阻塞，必须在 spawn_blocking 中调用）
///
/// 注意：gzip stdin 的写入必须与 stdout 的读取并发进行。真实归档压缩后约 14 MiB、
/// 解压后约 74 MiB，远超管道缓冲区（通常 64 KiB）。若先 `write_all` 再
/// `wait_with_output`，子进程阻塞在写 stdout、父进程阻塞在写 stdin，形成双向死锁，
/// 表现为「starting new connection 之后永久无输出」，且不受 HTTP 超时约束。
fn decompress_gzip(bytes: Vec<u8>) -> AppResult<String> {
    // 预览只取前 256 字节，避免对十几 MiB 的压缩数据做整体 UTF-8 转换
    let head_preview = truncate(
        &String::from_utf8_lossy(&bytes[..bytes.len().min(256)]),
        200,
    );

    let mut child = Command::new("gzip")
        .arg("-dc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::SystemCommand(format!("启动 gzip 失败: {}", e)))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| AppError::SystemCommand("无法获取 gzip stdin".to_string()))?;
    // 独立线程写 stdin；wait_with_output 并发读 stdout/stderr，避免管道死锁
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));

    let output = child
        .wait_with_output()
        .map_err(|e| AppError::SystemCommand(format!("等待 gzip 结束失败: {}", e)))?;
    let write_result = writer
        .join()
        .map_err(|_| AppError::SystemCommand("gzip stdin 写入线程异常".to_string()))?;

    if !output.status.success() {
        // Anubis 等反爬中间件返回 200 + HTML，gzip 会报「格式错误」，
        // 此时必须区分「被拦截」与「普通解析失败」，否则错误原因不可定位。
        if head_preview.trim_start().starts_with('<') {
            return Err(AppError::NetworkError(format!(
                "AUR 元数据归档疑似被反爬中间件拦截（返回 HTML 而非 gzip）：{}",
                head_preview
            )));
        }
        return Err(AppError::ParseError(format!(
            "gzip 解压失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    // 解压成功时写入失败才是真错误；gzip 提前失败时的 BrokenPipe 属预期，由上面分支处理
    write_result.map_err(|e| AppError::SystemCommand(format!("写入 gzip stdin 失败: {}", e)))?;
    String::from_utf8(output.stdout)
        .map_err(|e| AppError::ParseError(format!("归档内容非 UTF-8: {}", e)))
}

/// 保留 parse_aur_fields 所需字段，丢弃其余以控制缓存内存
fn slim_entry(item: &Value) -> Value {
    let mut map = serde_json::Map::new();
    for key in KEPT_FIELDS {
        if let Some(value) = item.get(*key) {
            map.insert((*key).to_string(), value.clone());
        }
    }
    Value::Object(map)
}

/// 由归档 results 数组建立 pkgname -> 精简包信息 的索引
fn build_index(results: &[Value]) -> HashMap<String, Value> {
    let mut index = HashMap::with_capacity(results.len());
    for item in results {
        if let Some(name) = item["Name"].as_str() {
            index.insert(name.to_string(), slim_entry(item));
        }
    }
    index
}

/// 解析归档字节流为按包名索引的映射（同步，供 spawn_blocking 调用）
pub(super) fn parse_archive(bytes: &[u8]) -> AppResult<HashMap<String, Value>> {
    let text = decompress_gzip(bytes.to_vec())?;
    if text.trim_start().starts_with('<') {
        return Err(AppError::NetworkError(format!(
            "AUR 元数据归档疑似被反爬中间件拦截（返回 HTML）：{}",
            truncate(&text, 200)
        )));
    }
    let data: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::ParseError(format!("AUR 元数据归档 JSON 解析失败: {}", e)))?;

    // packages-meta-ext-v1.json.gz 顶层是 JSON 数组；RPC v5 响应才是 {"results":[...]}。
    // 早期单测使用伪造的对象格式，真实归档被误判为「缺少 results 数组」后静默降级。
    let results: &[Value] = match &data {
        Value::Array(arr) => arr,
        Value::Object(_) => data["results"]
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| AppError::ParseError("AUR 元数据归档缺少 results 数组".to_string()))?,
        _ => {
            return Err(AppError::ParseError(
                "AUR 元数据归档顶层既不是数组也不是对象".to_string(),
            ))
        }
    };
    if results.is_empty() {
        return Err(AppError::ParseError(
            "AUR 元数据归档 results 为空，拒绝建立空基线".to_string(),
        ));
    }
    Ok(build_index(results))
}

/// 按请求的包名顺序从索引中取出结果
pub(super) fn filter_by_names(index: &HashMap<String, Value>, pkgnames: &[String]) -> Vec<Value> {
    pkgnames
        .iter()
        .filter_map(|name| index.get(name).cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 用系统 gzip 生成测试用的 .gz 字节
    fn gzip_text(text: &str) -> Vec<u8> {
        let mut child = Command::new("gzip")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("gzip 不可用");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap().stdout
    }

    /// 真实归档格式：顶层 JSON 数组
    #[test]
    fn parse_archive_builds_name_index_with_kept_fields() {
        let json = r#"[
            {"Name":"yay","Version":"12.0-1","Depends":["git"],"License":["MIT"],"OutOfDate":null,"NumVotes":900},
            {"Name":"paru","Version":"2.0-1","OutOfDate":1760000000}]"#;
        let index = parse_archive(&gzip_text(json)).unwrap();
        assert_eq!(index.len(), 2);
        assert_eq!(index["yay"]["Version"], "12.0-1");
        assert_eq!(index["yay"]["Depends"][0], "git");
        assert!(index["yay"].get("NumVotes").is_none(), "无关字段应被裁掉");
        assert_eq!(index["paru"]["OutOfDate"], 1760000000);
    }

    /// 兼容 RPC v5 的 {"results":[...]} 形状
    #[test]
    fn parse_archive_accepts_rpc_v5_object_shape() {
        let json = r#"{"version":5,"type":"multiinfo","resultcount":1,
            "results":[{"Name":"yay","Version":"12.0-1"}]}"#;
        let index = parse_archive(&gzip_text(json)).unwrap();
        assert_eq!(index.len(), 1);
        assert_eq!(index["yay"]["Version"], "12.0-1");
    }

    /// 解压输出远超管道缓冲区（64 KiB）时必须正常返回，
    /// 防止「先写完 stdin 再读 stdout」的双向死锁回归
    #[test]
    fn parse_archive_large_payload_no_pipe_deadlock() {
        let mut entries = Vec::new();
        for i in 0..2000 {
            entries.push(format!(
                r#"{{"Name":"pkg{i}","Version":"1.0-1","Description":"{pad}","Depends":["a","b"]}}"#,
                pad = "x".repeat(600)
            ));
        }
        let json = format!("[{}]", entries.join(","));
        assert!(json.len() > 1024 * 1024, "测试载荷应超过 1 MiB");
        let index = parse_archive(&gzip_text(&json)).unwrap();
        assert_eq!(index.len(), 2000);
        assert_eq!(index["pkg1999"]["Version"], "1.0-1");
    }

    #[test]
    fn parse_archive_rejects_anubis_html() {
        let err = parse_archive(&gzip_text("<!doctype html><html>Oh noes!</html>")).unwrap_err();
        assert!(err.to_string().contains("反爬"), "实际错误: {}", err);
        let err = parse_archive(b"<!doctype html><html>blocked</html>").unwrap_err();
        assert!(err.to_string().contains("反爬"), "实际错误: {}", err);
    }

    #[test]
    fn parse_archive_rejects_empty_results() {
        let err = parse_archive(&gzip_text(r#"{"version":5,"results":[]}"#)).unwrap_err();
        assert!(err.to_string().contains("空基线"), "实际错误: {}", err);
        let err = parse_archive(&gzip_text("[]")).unwrap_err();
        assert!(err.to_string().contains("空基线"), "实际错误: {}", err);
    }

    #[test]
    fn filter_by_names_keeps_only_requested_packages() {
        let json =
            r#"{"results":[{"Name":"yay","Version":"12.0-1"},{"Name":"paru","Version":"2.0-1"}]}"#;
        let data: Value = serde_json::from_str(json).unwrap();
        let index = build_index(data["results"].as_array().unwrap());
        let names = vec!["paru".to_string(), "missing".to_string()];
        let out = filter_by_names(&index, &names);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["Name"], "paru");
    }

    /// 真实归档端到端校验：设置 AUR_ARCHIVE_SAMPLE 指向 .gz 文件启用，CI 默认跳过。
    /// 用于验证数组格式、gzip 管道不死锁、索引规模合理。
    #[test]
    fn parse_archive_real_sample() {
        let Ok(path) = std::env::var("AUR_ARCHIVE_SAMPLE") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("读取归档样本失败");
        let index = parse_archive(&bytes).expect("真实归档解析失败");
        assert!(
            index.len() > 50_000,
            "真实归档条目数异常偏少: {}",
            index.len()
        );
    }
}
