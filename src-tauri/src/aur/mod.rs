/**
 * aur/mod.rs - AUR RPC API 交互模块
 *
 * 提供与 Arch Linux AUR (Arch User Repository) 交互的功能
 * 包括通过 RPC API 查询包信息和从本地 PKGBUILD 文件解析数据
 */
mod metadata; // AUR 全量元数据归档（批量基线）
mod pkgbuild; // PKGBUILD 文件解析子模块
mod rpc; // AUR RPC API 通信子模块
pub mod rpc_batch; // AUR RPC 批量 info 查询（分批、反爬识别、失败摘要）

pub use metadata::{get_packages_from_archive, get_packages_info_smart}; // 导出归档基线入口
pub use pkgbuild::{read_pkgbuild, sync_from_local_files}; // 导出 PKGBUILD 读取和本地同步函数
pub use rpc::{fetch_packages_by_user, get_package_info, search_packages, AurPackageData}; // 导出 AUR RPC 函数和数据结构
pub use rpc_batch::get_packages_info; // 导出 AUR RPC 批量查询入口
