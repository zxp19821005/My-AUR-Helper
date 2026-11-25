/**
 * helpers.rs - 版本检查器的结果构造与「检查完成」日志公共封装
 *
 * 功能：
 * - finish_check: 构造 CheckResult 并按各 checker 原有策略打「检查完成」日志
 * - MissingVersionLog: 显式承载各 checker 在「未找到上游版本」分支的日志差异
 *
 * 工作流程：
 * 1. 调用方把已求得的 version / license / language_names 显式传入，公共方法不自行补全；
 * 2. 有版本时统一打 info（文案逐字沿用 `[版本检查] 检查完成: {} -> 上游版本={}`）；
 * 3. 无版本时按 MissingVersionLog 变体决定 debug / 静默 / warn，文案与级别逐字沿用原实现；
 * 4. 返回显式构造的 CheckResult，避免 `..Default::default()` 吞掉 language_names 等特有字段。
 *
 * 适用场景：http / gitee / gitlab / redirect / browser / github tags_checker 六个检查器
 * 的收尾逻辑。api_checker 因多分支多来源（含 `(来自 tags)` 后缀）不纳入，保留原样。
 */
use log::{debug, info, warn};

use super::trait_def::CheckResult;

/// 「未找到上游版本」分支的日志策略。
///
/// 各 checker 原有实现并不一致，公共方法不得一刀切：
/// - http / browser / github tags_checker 打标准 debug 日志；
/// - gitee / gitlab 只有 info 分支，无 else；
/// - redirect 打一条带上游 URL 的 warn。
#[derive(Debug, Clone)]
pub(crate) enum MissingVersionLog {
    /// 标准 debug 日志：`[版本检查] 检查完成: {} -> 未找到上游版本`
    Debug,
    /// 不打日志（gitee / gitlab 原实现无 else 分支）
    Silence,
    /// redirect 专用 warn：`[版本检查] 未能从重定向 URL 提取版本: {}`
    WarnUrl(String),
}

/// 构造 `CheckResult` 并打「检查完成」日志。
///
/// 所有字段均由调用方显式传入，公共方法不使用 `Default` 补全，
/// 以保证 tags_checker 的 `language_names` 不会被吞掉。
///
/// # 参数
/// - `pkgname`: 软件包名称（用于日志）
/// - `version`: 检测到的上游版本，None 表示未找到
/// - `license`: License SPDX ID
/// - `language_names`: 编程语言名称列表
/// - `missing_log`: 未找到版本时的日志策略（见 `MissingVersionLog`）
///
/// # 返回
/// 字段完整的 `CheckResult`
pub(crate) fn finish_check(
    pkgname: &str,
    version: Option<String>,
    license: Option<String>,
    language_names: Vec<String>,
    missing_log: MissingVersionLog,
) -> CheckResult {
    if let Some(v) = &version {
        info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
    } else {
        match missing_log {
            MissingVersionLog::Debug => {
                debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
            }
            MissingVersionLog::Silence => {}
            MissingVersionLog::WarnUrl(url) => {
                warn!("[版本检查] 未能从重定向 URL 提取版本: {}", url);
            }
        }
    }
    CheckResult {
        version,
        license,
        language_names,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `finish_check` 必须把调用方传入的 language_names / license 原样带出，
    /// 不得被 `Default` 吞掉（tags_checker 的关键字段）。
    #[test]
    fn test_finish_check_preserves_all_fields() {
        let languages = vec!["Rust".to_string(), "TypeScript".to_string()];
        let result = finish_check(
            "demo",
            Some("1.2.3".to_string()),
            Some("MIT".to_string()),
            languages.clone(),
            MissingVersionLog::Debug,
        );
        assert_eq!(result.version.as_deref(), Some("1.2.3"));
        assert_eq!(result.license.as_deref(), Some("MIT"));
        assert_eq!(result.language_names, languages);
    }

    /// 无版本分支下三种日志策略都不改变结果字段，仅影响日志输出。
    #[test]
    fn test_finish_check_missing_version_keeps_languages() {
        let languages = vec!["Go".to_string()];
        for variant in [
            MissingVersionLog::Debug,
            MissingVersionLog::Silence,
            MissingVersionLog::WarnUrl("https://example.com".to_string()),
        ] {
            let result = finish_check("demo", None, None, languages.clone(), variant);
            assert!(result.version.is_none());
            assert!(result.license.is_none());
            assert_eq!(result.language_names, languages);
        }
    }
}
