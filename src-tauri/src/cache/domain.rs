/**
 * domain.rs - 缓存域定义
 *
 * 定义内存缓存支持的领域（Settings / Licenses / Languages）及其元信息。
 * 每个域对应一份内存条目；Licenses / Languages 为纯公开数据，支持落盘持久化，
 * Settings 含 token 等敏感信息，仅保留内存（不落盘，避免凭据明文写盘）。
 */
use serde::{Deserialize, Serialize};

/// 缓存域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheDomain {
    /// 系统设置（仅内存缓存，不落盘：可能含 token/secret 等敏感信息）
    Settings,
    /// License 枚举（公开数据，落盘）
    Licenses,
    /// 编程语言枚举（公开数据，落盘）
    Languages,
}

impl CacheDomain {
    /// 返回全部缓存域（用于遍历初始化 / 统计）
    pub fn all() -> [CacheDomain; 3] {
        [CacheDomain::Settings, CacheDomain::Licenses, CacheDomain::Languages]
    }

    /// 磁盘文件名（不含扩展名），白名单映射，杜绝路径注入
    pub fn file_name(self) -> &'static str {
        match self {
            CacheDomain::Settings => "settings",
            CacheDomain::Licenses => "licenses",
            CacheDomain::Languages => "languages",
        }
    }

    /// 是否支持磁盘持久化（Settings 因含敏感信息不落盘）
    pub fn persistent(self) -> bool {
        !matches!(self, CacheDomain::Settings)
    }

    /// 中文展示名（供统计接口与前端展示）
    pub fn label(self) -> &'static str {
        match self {
            CacheDomain::Settings => "系统设置",
            CacheDomain::Licenses => "License 枚举",
            CacheDomain::Languages => "编程语言枚举",
        }
    }
}
