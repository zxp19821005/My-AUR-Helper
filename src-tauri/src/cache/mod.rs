/**
 * mod.rs - 内存缓存模块
 *
 * 模块结构（各文件单一职责，均 <300 行）：
 * - domain.rs      — 缓存域定义（Settings / Licenses / Languages）与元信息
 * - config.rs      — 缓存配置（从 settings 表读取，含默认目录）
 * - manager.rs     — CacheManager：内存缓存主体（读取回源 / 失效 / 写盘 / 清空 / 统计）
 * - persistence.rs — 磁盘读写（原子写 + JSON 序列化）
 */
pub mod config;
pub mod domain;
pub mod manager;
pub mod persistence;

#[cfg(test)]
mod tests;

pub use config::CacheConfig;
pub use domain::CacheDomain;
pub use manager::CacheManager;
