/**
 * models/mod.rs - 数据模型模块
 *
 * 定义所有数据库表对应的 Rust 结构体
 * 每个结构体对应一个数据表，每个字段对应一列
 * 使用 serde 序列化/反序列化以支持 Tauri IPC 通信
 */

mod aur_info;                      // AUR 包信息模型
mod backup_software;               // 备份软件包模型
mod cache_software;                // 缓存软件包模型
mod checker_type;                  // 检查器类型枚举
mod enum_license;                  // License 枚举模型
mod enum_programming_language;     // 编程语言枚举模型
mod log_entry;                     // 日志条目模型
mod package_type;                  // 包类型枚举
mod proxy_info;                    // 代理信息模型
mod proxy_test;                    // 代理测试结果模型
mod proxy_type;                    // 代理类型枚举
mod setting;                       // 设置模型
mod software_info;                 // 软件包信息模型
mod software_list_entry;           // 软件包列表展示模型
mod upstream_info;                 // 上游版本信息模型

pub use aur_info::AurInfo;
pub use backup_software::BackupSoftware;
pub use cache_software::CacheSoftware;
pub use checker_type::CheckerType;
pub use enum_license::EnumLicense;
pub use enum_programming_language::EnumProgrammingLanguage;
pub use log_entry::LogEntry;
pub use package_type::PackageType;
pub use proxy_info::ProxyInfo;
pub use proxy_test::ProxyTest;
pub use proxy_type::ProxyType;
pub use setting::Setting;
pub use software_info::SoftwareInfo;
pub use software_list_entry::SoftwareListEntry;
pub use upstream_info::UpstreamInfo;
