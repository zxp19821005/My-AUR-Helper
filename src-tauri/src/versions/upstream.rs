use std::sync::OnceLock;

use super::remove_git_describe_metadata;
use log::debug;
use regex::Regex;

use super::rules::CleanupRules;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamVersion {
    pub raw: String,
    pub normalized_version: String,
}

/// 预编译正则：匹配版本前缀 v/V
fn re_v_prefix() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[vV]").unwrap())
}

/// 预编译正则：匹配发行版后缀
fn re_release_suffix() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"-(release|uos|arch|linux|debian|ubuntu|fedora|centos|el\d+|fc\d+|srpm|rpm|deb)$",
        )
        .unwrap()
    })
}

/// 预编译正则：匹配构建元数据
fn re_build_metadata() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\+[a-zA-Z0-9.-]+$").unwrap())
}

impl UpstreamVersion {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim().to_string();
        let normalized_version = Self::cleanup(&raw);

        debug!("解析上游版本 '{}': 标准化后='{}'", raw, normalized_version);

        UpstreamVersion {
            raw,
            normalized_version,
        }
    }

    pub fn parse_with_rules(raw: &str, rules: &CleanupRules) -> Self {
        let raw = raw.trim().to_string();
        let normalized_version = Self::cleanup_with_rules(&raw, rules);

        debug!(
            "使用规则解析上游版本 '{}': 标准化后='{}'",
            raw, normalized_version
        );

        UpstreamVersion {
            raw,
            normalized_version,
        }
    }

    fn cleanup(version: &str) -> String {
        let rules = CleanupRules::default();
        Self::cleanup_with_rules(version, &rules)
    }

    fn cleanup_with_rules(version: &str, rules: &CleanupRules) -> String {
        let mut result = version.to_string();

        result = Self::remove_prefixes(&result, &rules.prefixes);

        result = Self::remove_suffixes(&result, &rules.suffixes);

        // 使用公共函数清理 git describe 格式
        result = remove_git_describe_metadata(&result);

        result = Self::remove_build_metadata(&result);

        result = result.replace('-', "_");

        result = result.trim().to_string();

        if result.is_empty() {
            version.to_string()
        } else {
            result
        }
    }

    fn remove_prefixes(version: &str, prefixes: &[String]) -> String {
        let mut result = version.to_string();
        for prefix in prefixes {
            if result.starts_with(prefix) {
                result = result[prefix.len()..].to_string();
                break;
            }
        }
        result = re_v_prefix().replace(&result, "").to_string();
        result
    }

    fn remove_suffixes(version: &str, suffixes: &[String]) -> String {
        let mut result = version.to_string();
        for suffix in suffixes {
            if result.ends_with(suffix) {
                result = result[..result.len() - suffix.len()].to_string();
                break;
            }
        }

        result = re_release_suffix().replace(&result, "").to_string();

        result
    }

    fn remove_build_metadata(version: &str) -> String {
        re_build_metadata().replace(version, "").to_string()
    }
}

impl std::fmt::Display for UpstreamVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::rules::CleanupRules;

    #[test]
    fn test_parse_basic() {
        let v = UpstreamVersion::parse("1.2.3");
        assert_eq!(v.raw, "1.2.3");
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_with_v_prefix() {
        let v = UpstreamVersion::parse("v1.2.3");
        assert_eq!(v.raw, "v1.2.3");
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_with_v_prefix_uppercase() {
        let v = UpstreamVersion::parse("V2.4.6");
        assert_eq!(v.raw, "V2.4.6");
        assert_eq!(v.normalized_version, "2.4.6");
    }

    #[test]
    fn test_parse_with_release_suffix() {
        let v = UpstreamVersion::parse("1.2.3-release");
        assert_eq!(v.raw, "1.2.3-release");
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_with_uos_suffix() {
        let v = UpstreamVersion::parse("2.4.5-uos");
        assert_eq!(v.raw, "2.4.5-uos");
        assert_eq!(v.normalized_version, "2.4.5");
    }

    #[test]
    fn test_parse_with_linux_suffix() {
        let v = UpstreamVersion::parse("3.0.0-linux");
        assert_eq!(v.raw, "3.0.0-linux");
        assert_eq!(v.normalized_version, "3.0.0");
    }

    #[test]
    fn test_parse_hyphen_to_underscore() {
        let v = UpstreamVersion::parse("1.2.3-beta1");
        assert_eq!(v.raw, "1.2.3-beta1");
        assert_eq!(v.normalized_version, "1.2.3_beta1");
    }

    #[test]
    fn test_parse_build_metadata() {
        let v = UpstreamVersion::parse("1.2.3+build123");
        assert_eq!(v.raw, "1.2.3+build123");
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_complex() {
        let v = UpstreamVersion::parse("v2.4.5-alpha1-release");
        assert_eq!(v.raw, "v2.4.5-alpha1-release");
        assert_eq!(v.normalized_version, "2.4.5_alpha1");
    }

    #[test]
    fn test_parse_with_custom_rules() {
        let mut rules = CleanupRules::default();
        rules.add_prefix("myapp-");
        rules.add_suffix("-custom");

        let v = UpstreamVersion::parse_with_rules("myapp-1.2.3-custom", &rules);
        assert_eq!(v.raw, "myapp-1.2.3-custom");
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_git_describe_format() {
        // 测试 git describe 格式的版本
        let v = UpstreamVersion::parse("v2.0.1.r0.g30a6260");
        assert_eq!(v.raw, "v2.0.1.r0.g30a6260");
        assert_eq!(v.normalized_version, "2.0.1");

        let v = UpstreamVersion::parse("1.7.0.r0.gb9a08cc");
        assert_eq!(v.raw, "1.7.0.r0.gb9a08cc");
        assert_eq!(v.normalized_version, "1.7.0");

        let v = UpstreamVersion::parse("2.0.1.r5.g9a27946");
        assert_eq!(v.raw, "2.0.1.r5.g9a27946");
        assert_eq!(v.normalized_version, "2.0.1");
    }
}
