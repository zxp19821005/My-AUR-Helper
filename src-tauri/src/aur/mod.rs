/**
 * aur/mod.rs - AUR RPC API 交互模块
 *
 * 提供与 Arch Linux AUR (Arch User Repository) 交互的功能
 * 包括通过 RPC API 查询包信息和从本地 PKGBUILD 文件解析数据
 */
mod pkgbuild; // PKGBUILD 文件解析子模块
mod rpc;      // AUR RPC API 通信子模块

pub use pkgbuild::{read_pkgbuild, sync_from_local_files}; // 导出 PKGBUILD 读取和本地同步函数
pub use rpc::{fetch_packages_by_user, get_package_info, get_packages_info, AurPackageData}; // 导出 AUR RPC 函数和数据结构