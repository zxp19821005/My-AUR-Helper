/**
 * utils.rs - 版本检查器通用工具函数模块入口（barrel）
 *
 * 功能：
 * - 本文件仅负责子模块声明与公开函数再导出，不包含任何具体实现
 * - 对外保持 `checkers::utils::*`（`crate::checkers::utils::extract_owner_repo`
 *   等）路径不变，调用方无需改动 import
 *
 * 工作流程：
 * 1. regex 子模块提供静态正则集合与自定义正则提取
 * 2. url 子模块提供 owner/repo 解析与 URL/HTML 版本提取
 * 3. version 子模块提供版本清洗与归一化
 * 4. tests 子模块承载单元测试（仅 cfg(test) 编译）
 *
 * 适用场景：所有检查器的版本提取/清洗，拆分只为控制单文件行数，语义零变化。
 */
mod regex;
mod url;
mod version;

#[cfg(test)]
mod tests;

pub use regex::extract_version_with_regex;
pub use url::{extract_owner_repo, extract_version_from_html, extract_version_from_url};
pub use version::{clean_version, normalize_extracted_version};
