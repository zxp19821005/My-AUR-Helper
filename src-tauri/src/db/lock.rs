/**
 * lock.rs - 数据库连接锁的统一获取入口
 *
 * 功能：
 * - lock_db: 以 `?` 统一获取受 Mutex 保护的状态，消除散落各命令的
 *   `map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))` 重复。
 *
 * 错误转换依赖 errors/db.rs 中已有的 `impl<T> From<PoisonError<T>> for AppError`：
 * 锁中毒时统一转换为 `AppError::DatabaseLocked`，因此调用方只需
 * `let db = lock_db(&state.db)?;`，无需手写 map_err。
 *
 * 适用场景：所有通过 `State<'_, AppState>` 注入 Mutex<Database> 的 Tauri 命令。
 */
use std::sync::{Mutex, MutexGuard};

use crate::errors::AppResult;

/// 获取受 Mutex 保护的状态锁，统一锁错误的转换格式。
///
/// 依赖 `From<PoisonError<T>> for AppError`，锁中毒时返回
/// `AppError::DatabaseLocked`；正常情况返回可用的锁守卫。
///
/// # 参数
/// - `db`: 受 `Mutex` 保护的状态（通常为 `AppState.db`）
///
/// # 返回
/// - `Ok(MutexGuard)`: 可用的锁守卫
/// - `Err(AppError::DatabaseLocked)`: 互斥锁中毒（曾持有者 panic）
pub fn lock_db<T>(db: &Mutex<T>) -> AppResult<MutexGuard<'_, T>> {
    Ok(db.lock()?)
}
