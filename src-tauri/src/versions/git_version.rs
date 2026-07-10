use regex::Regex;

/// 清理 git describe 格式的版本元数据
/// 将 tag.rN.gHASH 格式转换为纯 tag 格式
///
/// # 示例
/// - `1.0.0.beta.118.r0.gc044c59` -> `1.0.0.beta.118`
/// - `v2.0.1.r0.g30a6260` -> `v2.0.1`
/// - `1.7.0.r0.gb9a08cc` -> `1.7.0`
pub fn remove_git_describe_metadata(version: &str) -> String {
    // 匹配 git describe 格式：tag.rN.gHASH
    // 例如：2.0.1.r0.g30a6260 或 1.7.0.r0.gb9a08cc
    let re = Regex::new(r"\.r\d+\.g[a-f0-9]+$").unwrap();
    re.replace(version, "").to_string()
}

/// 从 rN.HASH 格式中提取 commit count
/// 用于没有 tag 的 -git 包版本比较
///
/// # 示例
/// - `r1234.abcdef0` -> Some(1234)
/// - `r0.abcdef0` -> Some(0)
/// - `1.2.3` -> None
pub fn extract_commit_count(version: &str) -> Option<u64> {
    // 匹配 rN.HASH 格式
    let re = Regex::new(r"^r(\d+)\.[a-f0-9]+$").unwrap();
    re.captures(version)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u64>().ok())
}

/// 判断是否为 rN.HASH 格式（没有 tag 的 git 版本）
pub fn is_r_format(version: &str) -> bool {
    // 要求 hash 至少 7 个字符（git short hash 标准长度）
    let re = Regex::new(r"^r\d+\.[a-f0-9]{7,}$").unwrap();
    re.is_match(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_git_describe() {
        assert_eq!(
            remove_git_describe_metadata("1.0.0.beta.118.r0.gc044c59"),
            "1.0.0.beta.118"
        );
        assert_eq!(remove_git_describe_metadata("v2.0.1.r0.g30a6260"), "v2.0.1");
        assert_eq!(remove_git_describe_metadata("1.7.0.r0.gb9a08cc"), "1.7.0");
        assert_eq!(remove_git_describe_metadata("2.0.1.r5.g9a27946"), "2.0.1");
        // 非 git describe 格式应该保持不变
        assert_eq!(remove_git_describe_metadata("1.2.3"), "1.2.3");
        assert_eq!(remove_git_describe_metadata("1.2.3-beta1"), "1.2.3-beta1");
    }

    #[test]
    fn test_extract_commit_count() {
        assert_eq!(extract_commit_count("r1234.abcdef0"), Some(1234));
        assert_eq!(extract_commit_count("r0.abcdef0"), Some(0));
        assert_eq!(extract_commit_count("r99999.abcdef0"), Some(99999));
        // 非 r 格式应该返回 None
        assert_eq!(extract_commit_count("1.2.3"), None);
        assert_eq!(extract_commit_count("v1.2.3"), None);
        assert_eq!(extract_commit_count("1.2.3.r0.gabcdef0"), None);
    }

    #[test]
    fn test_is_r_format() {
        assert!(is_r_format("r1234.abcdef0"));
        assert!(is_r_format("r0.abcdef0"));
        assert!(is_r_format("r99999.abcdef0"));
        // 非 r 格式应该返回 false
        assert!(!is_r_format("1.2.3"));
        assert!(!is_r_format("v1.2.3"));
        assert!(!is_r_format("1.2.3.r0.gabcdef0"));
        assert!(!is_r_format("r1234"));
        assert!(!is_r_format("r1234.abc")); // hash 太短
    }
}
