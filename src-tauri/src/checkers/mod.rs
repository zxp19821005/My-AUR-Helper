/**
 * mod.rs - 版本检查器模块入口
 *
 * 汇总所有检查器实现（GitHub / Gitee / GitLab / HTTP / 重定向 / 浏览器 /
 * 手动），并提供工厂函数 get_checker 与检查选项/结果类型。
 *
 * 模块设计原则：
 * - mod.rs 仅负责模块声明和导出，不包含具体实现
 * - 每个子文件负责单一检查器，保持代码可维护性
 * - 所有文件严格控制在 300 行以内
 */
mod browser;
mod factory;
mod gitee;
pub mod github;
mod gitlab;
/// 检查结果构造与「检查完成」日志公共封装
mod helpers;
mod http;
mod manual;
mod redirect;
/// HTTP 重定向检查器的 URL 解析与脚本扫描辅助函数
mod redirect_parse;
mod trait_def;
pub mod utils;
/// 版本字符串清洗（去文件扩展名 / 架构 / 发行版后缀）：从 utils.rs 抽出，避免单文件超限
mod version_extract_clean;

pub use factory::{get_checker, CheckerSettings};
pub use trait_def::{CheckOptions, CheckResult, VersionChecker};
