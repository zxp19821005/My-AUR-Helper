/**
 * app_state.rs - Tauri 应用状态与配置读取辅助
 *
 * 功能：
 * - AppState 承载数据库连接与内存缓存，供所有命令通过 State 注入
 * - CloseAction 保存窗口关闭动作配置（隐藏到托盘 / 直接退出）
 * - get_config_dir / get_setting_string 提供配置目录与字符串设置读取
 *
 * 工作流程：
 * 1. setup 阶段创建 AppState 并 app.manage，命令经 `use crate::AppState` 引用
 * 2. 窗口事件处理经 app_events 读取 CloseAction 决定隐藏还是退出
 *
 * 适用场景：全局状态定义与配置读取，从 lib.rs 迁出以降低入口文件行数。
 */
use std::path::PathBuf;
use std::sync::Mutex;

use crate::{cache, db};

/// 应用状态：数据库连接与内存缓存
pub struct AppState {
    /// 数据库连接（Mutex 保护，命令内加锁使用）
    pub db: Mutex<db::Database>,
    /// 内存缓存管理器
    pub memory_cache: Mutex<cache::CacheManager>,
}

/// 窗口关闭动作配置（字符串值：minimize_to_tray 等）
pub(crate) struct CloseAction(pub(crate) String);

/// 获取应用配置目录（基于 dirs::config_dir，失败回退当前目录）
pub(crate) fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.zxp19821005.aur-helper")
}

/// 读取字符串设置，缺失或读取失败时返回默认值。
///
/// # 参数
/// - `db`: 数据库引用
/// - `key`: 设置键
/// - `default`: 缺失时的默认值
///
/// # 返回
/// - 设置值或默认值
pub(crate) fn get_setting_string(db: &db::Database, key: &str, default: &str) -> String {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .unwrap_or_else(|| default.to_string())
}
