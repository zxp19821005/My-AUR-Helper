/**
 * checkers/utils/version.rs - 版本号清洗与归一化
 *
 * 功能：
 * - clean_version 去除应用名前缀与 v/V 前缀，保留 prerelease/build 元数据
 * - normalize_extracted_version 统一清洗正则/HTML/URL 提取出的原始版本串，
 *   对含噪声的整段匹配兜底提取首个语义版本号
 *
 * 工作流程：
 * 1. strip_file_extensions 剥离文件扩展名与架构/发行版后缀
 * 2. clean_version 去除前缀；is_pure_version 判断是否已是纯版本号
 * 3. 仍含噪声时 extract_first_version 从原文提取首个语义版本兜底
 *
 * 适用场景：所有检查器对上游版本的最终清洗，对外经 utils barrel 暴露路径不变。
 */
use std::sync::OnceLock;

use super::regex::static_regexes;
use crate::checkers::version_extract_clean::strip_file_extensions;

/// 清理版本号：提取纯版本号部分
///
/// 支持以下格式：
/// - `v1.2.3` / `V1.2.3` -> `1.2.3`
/// - `appname-v1.2.3` -> `1.2.3`
/// - `appname-1.2.3` -> `1.2.3`
/// - `1.2.3` -> `1.2.3`
/// - `continuous` / `latest` -> 保持原样（无数字的 tag）
///
/// @param ver - 原始版本字符串
/// @returns 清理后的版本号，如果无法提取版本则返回原始字符串
pub fn clean_version(ver: &str) -> String {
    // 尝试匹配 appname-vX.Y.Z 或 appname-X.Y.Z 模式
    // 复用惰性编译的静态正则提取 v 或数字开头的版本部分
    let re = &static_regexes().clean;
    if let Some(cap) = re.captures(ver) {
        if let Some(version) = cap.get(1) {
            let version_str = version.as_str();
            // 去除开头的 v/V 前缀
            return version_str
                .trim_start_matches('v')
                .trim_start_matches('V')
                .to_string();
        }
    }
    // 回退：只去除开头的 v/V 前缀
    ver.trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

/// 统一清洗「页面/文本」检查器从正则、HTML 或 URL 提取出的原始版本串。
///
/// 分三步：
/// 1. 剥离文件扩展名与架构/平台/发行版后缀（`strip_file_extensions`）；
/// 2. 去除应用名前缀与 v/V 前缀（`clean_version`）；
/// 3. 若清洗后仍残留 app 名 / 构建号等噪声（用户正则无捕获组、匹配到
///    `Moneydance 2024.4 (5253` 之类整段），再从原文提取首个语义版本号兜底。
///
/// 这样即便用户没写捕获组，`helio-\d\.\d+-x64.deb`、`Moneydance\ \d.+\d+`
/// 这类正则也能分别得到 `3.18` / `2024.4`。此前 http/browser 检查器直接返回
/// `extract_version_with_regex` 的整段匹配，导致前缀/后缀泄漏进上游版本造成误报。
///
/// 对已是干净版本号的输入（如 `3.18`、`1.0.0-alpha.16`）幂等，不会破坏既有结果。
///
/// @param ver - 原始提取版本字符串
/// @returns 归一化后的版本字符串
pub fn normalize_extracted_version(ver: &str) -> String {
    let stripped = strip_file_extensions(ver);
    let cleaned = clean_version(&stripped);
    if is_pure_version(&cleaned) {
        return cleaned;
    }
    // 清洗后仍含噪声（app 名 / 构建号等）：从原文提取首个语义版本号兜底。
    // 用原文而非 cleaned，避免 clean 阶段误伤可能需要的数字段。
    if let Some(v) = extract_first_version(ver) {
        return v;
    }
    cleaned
}

/// 判断字符串是否已是「纯版本号」：仅含可选 v 前缀、点分数字、prerelease
/// （-alpha 等）与 pkgrel（-N）/ build（+meta）。用于决定是否需要进一步
/// 从噪声文本中提取版本。
fn is_pure_version(s: &str) -> bool {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"^v?\d[\w.+-]*$").expect("正则 is_pure_version 编译失败")
    });
    re.is_match(s)
}

/// 从可能含噪声的文本中提取首个语义版本号（v? + 至少两段点分数字）。
/// 用于「无捕获组正则匹配到整段带前缀/后缀文本」时的兜底提取，例如
/// `Moneydance 2024.4 (5253` 提取出 `2024.4`。要求至少 MAJOR.MINOR 两段，
/// 避免把单独的构建号 `5253` 误当成版本。
fn extract_first_version(ver: &str) -> Option<String> {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"v?\d+\.\d+(?:\.\d+)*").expect("正则 extract_first_version 编译失败")
    });
    re.find(ver)
        .map(|m| m.as_str().trim_start_matches('v').to_string())
}
