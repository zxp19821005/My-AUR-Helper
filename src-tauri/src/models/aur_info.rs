use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// AUR 包信息
/// 对应数据库 aur_info 表，存储从 AUR RPC API 获取的详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurInfo {
    /// 关联的软件包 ID（主键）
    pub software_id: i64,
    /// 软件包描述
    pub pkgdesc: Option<String>,
    /// AUR 中的当前版本号
    pub aur_version: Option<String>,
    /// 关联的 License 枚举 ID
    pub license_id: Option<i64>,
    /// AUR 数据最后更新时间
    pub last_updated: Option<NaiveDateTime>,
    /// 运行时依赖列表（JSON 数组字符串）
    pub depends: Option<String>,
    /// 构建依赖列表（JSON 数组字符串）
    pub makedepends: Option<String>,
    /// 可选依赖列表（JSON 数组字符串）
    pub optdepends: Option<String>,
    /// 提供的虚拟包（JSON 数组字符串）
    pub provides: Option<String>,
    /// 冲突的包（JSON 数组字符串）
    pub conflicts: Option<String>,
    /// 替换的包（JSON 数组字符串）
    pub replaces: Option<String>,
    /// AUR 投票数
    pub votes: Option<i64>,
    /// AUR 人气值（0~1 之间）
    pub popularity: Option<f64>,
    /// 是否被标记为过期
    pub out_of_date: Option<bool>,
    /// 提交者用户名
    pub submitted_by: Option<String>,
    /// 维护者列表（JSON 数组字符串）
    pub maintainers: Option<String>,
}
