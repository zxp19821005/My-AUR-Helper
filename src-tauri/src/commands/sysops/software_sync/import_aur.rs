//! import_aur.rs - 从 AUR 搜索并导入新软件包
//!
//! 功能：
//! - 搜索 AUR 中已存在但本地数据库未记录的软件包
//! - 将搜索结果展示给用户选择导入
//! - 导入时自动填充基本信息（上游URL、检查器类型等）
//!
//! 使用场景：
//! - 用户在 AUR 提交了新包，需要同步到本地管理
//! - 手动添加包时从 AUR 搜索参考信息

use log::info;
use tauri::State;

use super::super::proxy_utils::build_client;
use super::utils::read_http_timeout;
use crate::aur;
use crate::errors::AppResult;
use crate::models::{AurInfo, CheckerType, PackageType, SoftwareInfo};
use crate::AppState;

/// 从 AUR 搜索软件包（返回匹配结果列表）
///
/// # 参数
/// - `state`: Tauri 应用状态
/// - `search_keyword`: 搜索关键字（包名或部分名称）
///
/// # 返回
/// - `Ok(Vec<serde_json::Value>)`: 匹配的 AUR 包列表
#[tauri::command]
pub async fn search_aur_packages(
    state: State<'_, AppState>,
    search_keyword: String,
) -> AppResult<Vec<serde_json::Value>> {
    info!("[AUR搜索] 搜索关键词: {}", search_keyword);
    let timeout = {
        let db = state.db.lock()?;
        read_http_timeout(&db)
    };
    let client = build_client(timeout, false);

    // 调用 AUR RPC API 搜索
    let results = aur::search_packages(&client, &search_keyword).await?;

    info!("[AUR搜索] 找到 {} 个匹配结果", results.len());
    Ok(results)
}

/// 从 AUR 导入软件包到本地数据库
///
/// # 参数
/// - `state`: Tauri 应用状态
/// - `pkgname`: 要导入的包名
/// - `package_type`: 包类型（1=编译安装，2=二进制包，3=Git仓库，4=AppImage）
/// - `checker_type`: 检查器类型（1=GitHub Tag，2=GitHub Release，3=Gitee，4=GitLab，5=重定向，6=HTTP页面，7=手动，8=浏览器/JS渲染）
///
/// # 返回
/// - `Ok(software_id)`: 成功导入的软件包 ID
#[tauri::command]
pub async fn import_aur_package(
    state: State<'_, AppState>,
    pkgname: String,
    package_type: i32,
    checker_type: i32,
) -> AppResult<i64> {
    info!("[AUR导入] 导入包: {}", pkgname);

    // 先检查是否已存在（在 db lock 内完成）
    let exists = {
        let db = state.db.lock()?;
        db.get_software_by_name(&pkgname)?.is_some()
    };
    if exists {
        return Err(crate::errors::AppError::ConfigError(format!(
            "软件包 '{}' 已存在于本地数据库",
            pkgname
        )));
    }

    // 获取 timeout 后释放 db lock
    let timeout = {
        let db = state.db.lock()?;
        read_http_timeout(&db)
    };
    let client = build_client(timeout, false);

    // 从 AUR 获取完整信息（此期间 db lock 已释放，不会阻塞其他线程）
    let aur_data = match aur::get_package_info(&client, &pkgname).await? {
        Some(data) => data,
        None => {
            return Err(crate::errors::AppError::ConfigError(format!(
                "AUR 中未找到包 '{}'，请先在 AUR 提交或等待审核通过",
                pkgname
            )));
        }
    };

    // 解析 AUR 数据
    let pkgdesc = aur_data["Description"].as_str().unwrap_or("").to_string();
    let upstream_url = aur_data["URL"].as_str().map(|s| s.to_string()).or_else(|| {
        // 尝试从包名推断上游 URL
        if pkgname.ends_with("-bin") {
            let base = pkgname.trim_end_matches("-bin");
            if base == "electron" {
                return Some("https://github.com/electron/electron".to_string());
            }
        }
        None
    });

    // 推断检查器类型（根据包名后缀）
    let inferred_checker_type = if pkgname.ends_with("-git") {
        3 // Git仓库
    } else if pkgname.ends_with("-bin") || pkgname.ends_with("-appimage") {
        2 // 二进制包
    } else {
        checker_type // 使用传入的值
    };

    // 创建 SoftwareInfo
    let sw = SoftwareInfo {
        software_id: None,
        pkgname: pkgname.clone(),
        upstream_url,
        package_type_id: PackageType::from_id(package_type),
        checker_type_id: CheckerType::from_id(inferred_checker_type),
        is_outdated: false,
        check_test_versions: false,
        check_binary_files: pkgname.ends_with("-bin") || pkgname.ends_with("-appimage"),
        auto_check_enabled: true,
        skip_check_upstream: false,
        language_ids: Vec::new(),
        version_extract_regex: None,
    };

    // 插入数据库
    let db = state.db.lock()?;
    let id = db.insert_software(&sw)?;

    // 插入 AUR 信息
    let license = aur_data["License"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let depends = aur_data["Depends"].as_array().map(|a| {
        serde_json::to_string(
            &a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>(),
        )
        .unwrap_or_default()
    });
    let makedepends = aur_data["MakeDepends"].as_array().map(|a| {
        serde_json::to_string(
            &a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>(),
        )
        .unwrap_or_default()
    });
    let optdepends = aur_data["OptDepends"].as_array().map(|a| {
        serde_json::to_string(
            &a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>(),
        )
        .unwrap_or_default()
    });

    let aur_info = AurInfo {
        software_id: id,
        pkgdesc: Some(pkgdesc),
        aur_version: aur_data["Version"].as_str().map(|s| s.to_string()),
        license_id: license,
        last_updated: aur_data["LastModified"].as_i64(),
        depends,
        makedepends,
        optdepends,
        out_of_date: aur_data["OutOfDate"].as_i64().map(|v| v != 0),
        last_sync_error: None,
    };
    db.upsert_aur_info(&aur_info)?;

    info!("[AUR导入] 成功导入 '{}'，ID: {}", pkgname, id);
    Ok(id)
}
