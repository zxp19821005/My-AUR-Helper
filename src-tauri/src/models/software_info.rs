use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

use super::{CheckerType, PackageType}; // 导入类型枚举

/// 软件包信息
/// 对应数据库 software_info 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    /// 软件包 ID，数据库自增主键，新增时为 None
    pub software_id: Option<i64>,
    /// 软件包名称（唯一）
    pub pkgname: String,
    /// 上游项目地址，用于版本检查
    pub upstream_url: Option<String>,
    /// 软件包类型：编译/二进制/Git/AppImage
    pub package_type: PackageType,
    /// 版本检查器类型：GitHub/Gitee/GitLab/Redirect/Http/Manual
    pub checker_type: CheckerType,
    /// 是否有上游更新
    pub is_outdated: bool,
    /// 是否同时检查测试版本（如 beta/rc）
    pub check_test_versions: bool,
    /// 是否检查二进制文件差异
    pub check_binary_files: bool,
    /// 是否启用自动检查上游版本
    pub auto_check_enabled: bool,
    /// 关联的 License 枚举 ID
    pub license_id: Option<i64>,
    /// 关联的编程语言枚举 ID
    pub language_id: Option<i64>,
    /// 创建时间（Unix 时间戳）
    pub created_at: i64,
}
