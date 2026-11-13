//! software_update/mod.rs - 软件更新命令模块
//!
//! 本模块负责「软件更新」工作台的后端能力：
//! - list: 汇总 AUR / 上游 / 本地 PKGBUILD 三方版本，推导更新状态（只读）
//! - apply: dry-run 预览变更与批量写回 PKGBUILD（写前备份）
//! - pkgbuild_io: 包名校验、路径解析、pkgver / pkgrel 改写与备份工具
//!
//! 模块设计原则：
//! - mod.rs 仅负责模块声明与导出，不含具体实现
//! - 不新增数据库表、不改 schema，状态由磁盘实际 pkgver 实时推导
//! - 所有写操作必须先经 preview 的 diff 预览，由前端取得用户确认
//! - 每个文件严格控制在 300 行以内

/// 版本汇总与状态推导（lib.rs 以完整路径注册命令，故需 pub）
pub mod list;

/// dry-run 预览与批量写回（lib.rs 以完整路径注册命令，故需 pub）
pub mod apply;

/// 内部 PKGBUILD 读写工具，不对外暴露
mod pkgbuild_io;

pub use apply::{apply_software_updates, preview_software_updates};
pub use list::list_software_updates;
