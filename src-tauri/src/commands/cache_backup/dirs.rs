/**
 * dirs.rs - 缓存目录通用工具
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

/// 在缓存目录中查找文件
pub async fn find_cache_file(filename: &str, cache_dirs: &[CacheDir]) -> Option<std::path::PathBuf> {
    for dir in cache_dirs {
        let path = std::path::Path::new(&dir.path);
        let file_path = path.join(filename);
        if file_path.exists() {
            return Some(file_path);
        }
    }
    None
}
