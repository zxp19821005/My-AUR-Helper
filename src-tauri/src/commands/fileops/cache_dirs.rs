/**
 * cache_dirs.rs - 缓存目录通用工具
 *
 * 功能：
 * - 缓存目录结构体定义
 * - 路径展开（将 ~ 展开为用户主目录）
 * - 从 settings 表获取启用的缓存目录列表
 * - 从缓存文件名提取包名
 * - 在缓存目录中查找文件
 */
use crate::errors::AppResult;

/// 缓存目录配置
pub struct CacheDir {
    pub name: String,
    pub path: String,
}

/// 将路径中的 ~ 展开为用户主目录
pub fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// 从 settings 表获取启用的缓存目录列表
pub fn get_cache_dirs(db: &crate::db::Database) -> AppResult<Vec<CacheDir>> {
    let mut dirs = Vec::new();

    // 系统缓存
    if let Some(system_dir) = db.get_setting("cache_dir_system")? {
        let system_enabled = db
            .get_setting("cache_dir_system_enabled")?
            .map(|s| s.value != "false")
            .unwrap_or(true);

        if system_enabled && !system_dir.value.is_empty() {
            dirs.push(CacheDir {
                name: "系统缓存".to_string(),
                path: expand_tilde(&system_dir.value),
            });
        }
    }

    // paru 缓存
    if let Some(paru_dir) = db.get_setting("cache_dir_paru")? {
        let paru_enabled = db
            .get_setting("cache_dir_paru_enabled")?
            .map(|s| s.value != "false")
            .unwrap_or(true);

        if paru_enabled && !paru_dir.value.is_empty() {
            dirs.push(CacheDir {
                name: "paru 缓存".to_string(),
                path: expand_tilde(&paru_dir.value),
            });
        }
    }

    // yay 缓存
    if let Some(yay_dir) = db.get_setting("cache_dir_yay")? {
        let yay_enabled = db
            .get_setting("cache_dir_yay_enabled")?
            .map(|s| s.value != "false")
            .unwrap_or(true);

        if yay_enabled && !yay_dir.value.is_empty() {
            dirs.push(CacheDir {
                name: "yay 缓存".to_string(),
                path: expand_tilde(&yay_dir.value),
            });
        }
    }

    // 自定义缓存目录
    if let Some(custom_dirs) = db.get_setting("cache_dirs_custom")? {
        if !custom_dirs.value.is_empty() {
            if let Ok(custom_list) =
                serde_json::from_str::<Vec<serde_json::Value>>(&custom_dirs.value)
            {
                for dir in custom_list {
                    let name = dir
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("自定义缓存")
                        .to_string();
                    let path = dir.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let is_enabled = dir
                        .get("is_enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    if is_enabled && !path.is_empty() {
                        dirs.push(CacheDir {
                            name,
                            path: expand_tilde(path),
                        });
                    }
                }
            }
        }
    }

    Ok(dirs)
}

/// 从缓存文件名提取包名
pub fn extract_pkgname_from_cache(filename: &str) -> Option<String> {
    let base = filename.strip_suffix(".pkg.tar.zst")?;
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    let name_ver = parts[2];
    let dash_pos = name_ver.rfind('-')?;
    Some(name_ver[..dash_pos].to_string())
}

/// 在缓存目录中查找文件（递归搜索子目录）
pub async fn find_cache_file(
    filename: &str,
    cache_dirs: &[CacheDir],
) -> Option<std::path::PathBuf> {
    for dir in cache_dirs {
        let root = std::path::Path::new(&dir.path);
        if let Some(found) = find_file_recursive(root, filename).await {
            return Some(found);
        }
    }
    None
}

/// 递归遍历目录查找文件名匹配的文件
///
/// 使用显式栈深度优先遍历；通过 `file_type()` 判断目录类型，
/// 对符号链接目录返回 false，天然排除符号链接、避免死循环。
async fn find_file_recursive(dir: &std::path::Path, filename: &str) -> Option<std::path::PathBuf> {
    let mut stack: Vec<std::path::PathBuf> = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(&current).await {
            Ok(e) => e,
            Err(_) => continue, // 无权限或目录不存在，跳过该分支
        };
        loop {
            let entry = match entries.next_entry().await {
                Ok(Some(e)) => e,
                _ => break,
            };
            let path = entry.path();
            let ft = match entry.file_type().await {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if ft.is_dir() {
                // file_type().is_dir() 对符号链接为 false，天然排除符号链接目录
                stack.push(path);
            } else if ft.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
                    return Some(path);
                }
            }
        }
    }
    None
}
