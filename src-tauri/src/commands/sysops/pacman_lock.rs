/**
 * pacman_lock.rs - pacman 写事务全局串行化锁
 *
 * 功能：
 * - 提供 `with_pacman_write_lock`，保证任意时刻全局只有一个 pacman 写事务在运行。
 *
 * 背景：
 * pacman 同一时刻只允许一个事务持有 `/var/lib/pacman/db.lck` 数据库锁。
 * 备份安装（`install_backup_package`）与缓存安装（`install_cache_package`）都通过
 * `sudo pacman -U` 写数据库。若两者并发、或同一安装被 UI 重复触发，
 * 会互相争抢 db.lck，导致「无法锁定数据库：文件已存在」失败。
 *
 * 该锁在后端兜底串行化所有 pacman 写操作，无论触发来源是否并发都能避免锁冲突。
 * 读操作（如 `pacman -Qip`）使用共享读锁，不受此锁影响，无需纳入。
 */
use std::future::Future;
use std::sync::OnceLock;
use tokio::sync::Mutex;

/// 全局 pacman 写操作锁（进程级，懒初始化）。
static PACMAN_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// 在 pacman 写锁保护下执行给定异步操作，保证任意时刻仅有一个 pacman 写事务运行。
///
/// 锁会在整个 `op` Future 执行期间持有（含子进程 `sudo pacman -U` 的等待），
/// 从而杜绝并发写事务争抢 `db.lck`。
///
/// @param op - 返回 Future 的闭包，内部应执行 `sudo pacman -U` 等写操作
/// @returns op 的执行结果（透传）
pub async fn with_pacman_write_lock<F, Fut, T>(op: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let lock = PACMAN_WRITE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().await;
    op().await
}
