/**
 * mod.rs - 文件操作命令模块入口
 *
 * 本模块包含所有与文件操作相关的 Tauri 命令：
 * - operations: 文件和目录的增删改查操作
 * - scan: 包文件扫描功能
 *
 * 模块设计原则：
 * - mod.rs 仅负责模块声明和导出，不包含具体实现
 * - 每个子文件负责单一功能，保持代码可维护性
 * - 所有文件严格控制在 300 行以内
 */

/// 文件和目录的增删改查操作
pub mod operations;

/// 包文件扫描功能
pub mod scan;

// 公开导出 Tauri 命令函数，供 lib.rs 注册使用
pub use operations::copy_file;
pub use operations::move_file;
pub use operations::delete_file;
pub use operations::delete_directory;
pub use operations::create_directory;
pub use operations::read_file;
pub use operations::list_directory;
pub use operations::file_exists;
pub use operations::file_metadata;
pub use operations::batch_delete;
pub use scan::scan_pkg_files;