/**
 * checkers/utils/regex.rs - 版本提取静态正则集合与自定义正则提取
 *
 * 功能：
 * - 进程级惰性编译一次静态正则集合（clean / url_v / url_num / html_kw / html_td），
 *   供 URL 提取与版本清洗子模块共享，避免每次提取都重新编译正则
 * - 提供基于调用方传入的自定义正则从文本中提取版本号的纯函数
 *
 * 工作流程：
 * 1. static_regexes() 首次调用时用 OnceLock 编译并缓存 StaticRegexes 单例
 * 2. url.rs / version.rs 通过 super::regex::static_regexes() 复用同一份集合
 * 3. extract_version_with_regex() 每次按传入模式即时编译并提取首个捕获组
 *
 * 适用场景：所有需要正则版本提取的检查器工具，对外经 utils barrel 暴露路径不变。
 */
use std::sync::OnceLock;

use log::debug;

/// 版本提取所需的静态正则集合（进程级单例，字段对兄弟子模块可见）
pub(super) struct StaticRegexes {
    /// 剥离应用名前缀、提取 v 或数字开头的版本
    pub(super) clean: regex::Regex,
    /// URL 中带 v 前缀的版本号
    pub(super) url_v: regex::Regex,
    /// URL 中不带 v 前缀的版本号
    pub(super) url_num: regex::Regex,
    /// HTML 中关键词（version/release/版本）后的版本号
    pub(super) html_kw: regex::Regex,
    /// HTML 表格 <td> 内的版本号
    pub(super) html_td: regex::Regex,
}

/// 获取版本提取静态正则集合单例（首次调用时惰性编译）
pub(super) fn static_regexes() -> &'static StaticRegexes {
    static RE: OnceLock<StaticRegexes> = OnceLock::new();
    RE.get_or_init(|| StaticRegexes {
        clean: regex::Regex::new(r"^[^0-9]*?(v?\d\S*)$").expect("正则 clean 编译失败"),
        url_v: regex::Regex::new(r"[/-]v?(\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)")
            .expect("正则 url_v 编译失败"),
        url_num: regex::Regex::new(r"[/-](\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)")
            .expect("正则 url_num 编译失败"),
        html_kw: regex::Regex::new(
            r"(?i)(?:version|release|ver\.?|版本)[:\s]+v?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)",
        )
        .expect("正则 html_kw 编译失败"),
        html_td: regex::Regex::new(
            r"(?i)(?:v)?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)(?:\s*</[aA]>)?\s*</[tT][dD]>\s*<[tT][dD]",
        )
        .expect("正则 html_td 编译失败"),
    })
}

/// 使用自定义正则表达式从文本中提取版本号
/// @param text - 包含版本号的文本
/// @param regex_pattern - 正则表达式模式
/// @returns 提取到的版本号，如果匹配失败或正则无效则返回 None
pub fn extract_version_with_regex(text: &str, regex_pattern: &str) -> Option<String> {
    debug!("[正则提取] 使用正则表达式: {}", regex_pattern);
    match regex::Regex::new(regex_pattern) {
        Ok(re) => {
            if let Some(caps) = re.captures(text) {
                let version = if caps.len() > 1 {
                    caps[1].to_string()
                } else {
                    caps[0].to_string()
                };
                debug!("[正则提取] 提取成功: {}", version);
                Some(version)
            } else {
                debug!("[正则提取] 未匹配到任何内容");
                None
            }
        }
        Err(e) => {
            debug!("[正则提取] 正则表达式无效: {}", e);
            None
        }
    }
}
