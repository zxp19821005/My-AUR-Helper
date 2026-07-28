use crate::db::Database;
use crate::errors::AppResult;
/**
 * dedup.rs - 备份去重逻辑
 *
 * 功能：
 * - collect_pkg_map: 按包名分组所有备份记录
 * - collect_files_to_delete: 比较版本，收集需要删除的旧版本文件
 * - DeduplicateResult: 去重结果结构体
 */
use log::info;

/// 备份去重结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeduplicateResult {
    pub removed_files: usize,
    pub removed_records: usize,
    pub errors: Vec<String>,
}

/// 解析 .pkg.tar.zst 文件名（提取包名、epoch、版本、pkgrel、架构）
pub fn parse_pkg_filename(filename: &str) -> Option<(String, i64, String, String, String)> {
    let base = filename.strip_suffix(".pkg.tar.zst")?;
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    let arch = parts[0].to_string();
    let pkgrel = parts[1].to_string();
    let name_ver = parts[2];
    let dash_pos = name_ver.rfind('-')?;
    let name = name_ver[..dash_pos].to_string();
    let ver_part = name_ver[dash_pos + 1..].to_string();
    let (epoch, version) = if let Some(pos) = ver_part.find(':') {
        (
            ver_part[..pos].parse::<i64>().unwrap_or(0),
            ver_part[pos + 1..].to_string(),
        )
    } else {
        (0, ver_part)
    };
    Some((name, epoch, version, pkgrel, arch))
}

/// 从数据库读取所有备份记录并按包名分组
pub fn collect_pkg_map(
    db: &Database,
) -> AppResult<std::collections::HashMap<String, Vec<(i64, String, String, String)>>> {
    let entries = db.get_all_backup_software()?;
    let mut pkg_map: std::collections::HashMap<String, Vec<(i64, String, String, String)>> =
        std::collections::HashMap::new();

    for entry in &entries {
        if let Some(id) = entry.id {
            if let Some((name, epoch, version, pkgrel, _arch)) = parse_pkg_filename(&entry.filename)
            {
                let full_ver = if epoch > 0 {
                    format!("{}:{}-{}", epoch, version, pkgrel)
                } else {
                    format!("{}-{}", version, pkgrel)
                };
                pkg_map.entry(name).or_default().push((
                    id,
                    entry.filename.clone(),
                    full_ver,
                    entry.full_path.clone(),
                ));
            }
        }
    }
    Ok(pkg_map)
}

/// 比较版本，收集需要删除的旧版本文件
pub fn collect_files_to_delete(
    pkg_map: &std::collections::HashMap<String, Vec<(i64, String, String, String)>>,
) -> Vec<(String, i64, String)> {
    let mut files_to_delete = Vec::new();
    let compare_versions = crate::versions::comparison::compare_versions;

    for (_pkg_name, entries) in pkg_map {
        if entries.len() <= 1 {
            continue;
        }
        let mut best_idx = 0;
        for i in 1..entries.len() {
            let cmp = compare_versions(&entries[i].2, &entries[best_idx].2);
            if cmp == crate::versions::comparison::VersionComparison::GreaterThan {
                best_idx = i;
            }
        }
        for (i, (id, filename, _ver, full_path)) in entries.iter().enumerate() {
            if i != best_idx {
                files_to_delete.push((filename.clone(), *id, full_path.clone()));
                info!(
                    "[备份管理] 标记删除旧版本: {} (最新: {})",
                    filename, entries[best_idx].1
                );
            }
        }
    }

    info!(
        "[备份管理] 共 {} 个包存在多版本，需删除 {} 个旧文件",
        pkg_map.values().filter(|v| v.len() > 1).count(),
        files_to_delete.len()
    );

    files_to_delete
}
