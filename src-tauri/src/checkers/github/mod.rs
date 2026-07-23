/**
 * mod.rs - GitHub 检查器模块入口
 *
 * 本模块包含所有 GitHub 相关的版本检查功能：
 * - tags: 通过 GitHub API 获取 tags 列表并提取最新版本
 * - release: GitHub Release API 调用（latest + 分页遍历）
 * - binary_check: 二进制文件检查工具
 * - repo_info: 仓库元信息获取（License + 编程语言）
 * - git_describe: 为 -git 包生成类似 git describe 的版本字符串
 * - tags_checker: GitHubTagsChecker 检查器实现
 * - api_checker: GitHubAPIChecker 检查器实现
 *
 * 模块设计原则：
 * - mod.rs 仅负责模块声明和导出，不包含具体实现
 * - 每个子文件负责单一功能，保持代码可维护性
 * - 所有文件严格控制在 300 行以内
 */

/// GitHub Tags 分页获取和版本比较逻辑
mod tags;

/// GitHub Release API 调用和资产过滤逻辑
mod release;

/// 二进制文件检查工具
mod binary_check;

/// 仓库元信息获取（License + 编程语言）
mod repo_info;

/// Git Describe 格式化（-git 包专用版本生成）
mod git_describe;

/// GitHubTagsChecker 检查器实现
mod tags_checker;

/// GitHubAPIChecker 检查器实现
mod api_checker;

// 公开导出检查器结构体，供 checkers/mod.rs 中的工厂函数使用
pub use api_checker::GitHubAPIChecker;
pub use tags_checker::GitHubTagsChecker;
