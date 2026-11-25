/**
 * software.rs - 软件包相关 Tauri 命令
 *
 * 提供软件包列表查询、详情获取、删除等 IPC 命令，是前端软件管理页
 * 与数据库/业务逻辑之间的桥梁。
 */
use log::{debug, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
#[specta::specta]
pub async fn list_software(state: State<'_, AppState>) -> Result<Vec<SoftwareInfo>, String> {
    debug!("正在获取所有软件包列表");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_software().map_err(|e| e.to_string())?;
    info!("已列出 {} 个软件包", result.len());
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn list_software_view(
    state: State<'_, AppState>,
) -> Result<Vec<SoftwareListEntry>, String> {
    debug!("正在获取软件包视图列表");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_software_list_entries().map_err(|e| e.to_string())?;
    info!("已列出 {} 个软件包视图条目", result.len());
    Ok(result)
}

/// 按 pkgname 获取单条列表视图条目（定向刷新用）
#[tauri::command]
#[specta::specta]
pub async fn get_software_list_entry(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<Option<SoftwareListEntry>, String> {
    debug!("正在获取软件包列表条目: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_software_list_entry(&pkgname)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_software(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<Option<SoftwareInfo>, String> {
    debug!("正在获取软件包: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_software_by_name(&pkgname).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_software_detail(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<Option<SoftwareDetail>, String> {
    debug!("正在获取软件包详情: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_software_detail_by_name(&pkgname)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_prev_next_software(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<(Option<String>, Option<String>), String> {
    debug!("正在获取上一个/下一个软件包: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_prev_next_software(&pkgname)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn search_software(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<SoftwareInfo>, String> {
    debug!("正在搜索软件包: {}", keyword);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.search_software(&keyword).map_err(|e| e.to_string())?;
    info!("搜索 '{}' 找到 {} 个结果", keyword, result.len());
    Ok(result)
}

/// 新增/更新软件包的统一入参
///
/// 用单个 struct 承接 Tauri 命令参数，以规避 specta 对函数参数数量（≤10）的限制；
/// 字段用 camelCase 与前端 AddSoftwareInput/UpdateSoftwareInput 保持一致，
/// 前端仍是扁平 invoke 对象，调用方式不变。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareFormInput {
    /// 更新时的软件包 ID（新增忽略）
    pub software_id: Option<i64>,
    /// 包名
    pub pkgname: String,
    /// 上游版本检查地址
    pub upstream_url: Option<String>,
    /// 包类型 ID
    pub package_type: i32,
    /// 检查器类型 ID
    pub checker_type: i32,
    /// 是否过期（新增忽略）
    pub is_outdated: Option<bool>,
    /// 是否检查测试版本
    pub check_test_versions: bool,
    /// 是否检查二进制文件
    pub check_binary_files: bool,
    /// 是否自动检查
    pub auto_check_enabled: bool,
    /// 是否跳过上游检查
    pub skip_check_upstream: bool,
    /// 关联语言 ID 列表
    pub language_ids: Vec<i64>,
    /// 版本提取正则
    pub version_extract_regex: Option<String>,
}

impl SoftwareFormInput {
    /// 将入参转换为软件信息实体
    ///
    /// # 参数
    /// - `software_id`: 显式指定的 ID；None 时取入参内的 ID（新增场景）
    /// - `default_outdated`: 新增时 is_outdated 的默认值
    fn into_software(self, software_id: Option<i64>, default_outdated: bool) -> SoftwareInfo {
        SoftwareInfo {
            software_id,
            pkgname: self.pkgname,
            upstream_url: self.upstream_url,
            package_type_id: PackageType::from_id(self.package_type),
            checker_type_id: CheckerType::from_id(self.checker_type),
            is_outdated: self.is_outdated.unwrap_or(default_outdated),
            check_test_versions: self.check_test_versions,
            check_binary_files: self.check_binary_files,
            auto_check_enabled: self.auto_check_enabled,
            skip_check_upstream: self.skip_check_upstream,
            language_ids: self.language_ids,
            version_extract_regex: self.version_extract_regex,
        }
    }
}

/// 添加软件包（Tauri IPC 命令）
#[tauri::command]
#[specta::specta]
pub async fn add_software(
    state: State<'_, AppState>,
    input: SoftwareFormInput,
) -> Result<i64, String> {
    info!("正在添加软件包: {}", input.pkgname);
    let sw = input.into_software(None, false);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = db.insert_software(&sw).map_err(|e| e.to_string())?;
    info!("已添加软件包 '{}'，ID: {}", sw.pkgname, id);
    Ok(id)
}

/// 更新软件包（Tauri IPC 命令）
#[tauri::command]
#[specta::specta]
pub async fn update_software(
    state: State<'_, AppState>,
    input: SoftwareFormInput,
) -> Result<(), String> {
    let sid = input
        .software_id
        .ok_or_else(|| "缺少 softwareId".to_string())?;
    info!("正在更新软件包 {}: {}", sid, input.pkgname);
    let sw = input.into_software(Some(sid), false);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.upsert_software(&sw).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_software_license(
    state: State<'_, AppState>,
    software_id: i64,
    license_id: Option<String>,
) -> Result<(), String> {
    info!(
        "正在设置软件包 {} 的 License 为 {:?}",
        software_id, license_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_aur_license(software_id, license_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_software_language(
    state: State<'_, AppState>,
    software_id: i64,
    language_id: Option<i64>,
) -> Result<(), String> {
    info!(
        "正在设置软件包 {} 的编程语言为 {:?}",
        software_id, language_id
    );
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_software_language(software_id, language_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_software(state: State<'_, AppState>, software_id: i64) -> Result<(), String> {
    info!("正在删除软件包，ID: {}", software_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_software(software_id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn batch_delete_software(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> Result<i64, String> {
    info!("正在批量删除 {} 个软件包", pkgname_list.len());
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0i64;
    for pkgname in &pkgname_list {
        if let Some(sw) = db
            .get_software_by_name(pkgname)
            .map_err(|e| e.to_string())?
        {
            if let Some(id) = sw.software_id {
                if db.delete_software(id).is_ok() {
                    count += 1;
                }
            }
        }
    }
    info!("已删除 {} 个软件包", count);
    Ok(count)
}
