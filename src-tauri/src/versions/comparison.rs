use log::debug;
use super::git_version::{extract_commit_count, is_r_format};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    LessThan,
    GreaterThan,
    Equal,
    Incomparable,
}

pub fn compare_vercmp(a: &str, b: &str) -> VersionComparison {
    // 将 _ 统一替换为 . 以便正确分割组件
    let a = a.replace('_', ".").trim().to_string();
    let b = b.replace('_', ".").trim().to_string();

    if a == b {
        return VersionComparison::Equal;
    }

    // 特殊处理 rN.HASH 格式（没有 tag 的 git 版本）
    if is_r_format(&a) && is_r_format(&b) {
        let a_count = extract_commit_count(&a);
        let b_count = extract_commit_count(&b);
        
        match (a_count, b_count) {
            (Some(a_count), Some(b_count)) => {
                debug!("[版本比较] r格式比较: AUR commit_count={} vs 上游 commit_count={}", a_count, b_count);
                return match a_count.cmp(&b_count) {
                    std::cmp::Ordering::Less => VersionComparison::LessThan,
                    std::cmp::Ordering::Greater => VersionComparison::GreaterThan,
                    std::cmp::Ordering::Equal => VersionComparison::Equal,
                };
            }
            _ => return VersionComparison::Incomparable,
        }
    }

    let (a_epoch, a_rest) = split_epoch(&a);
    let (b_epoch, b_rest) = split_epoch(&b);

    if let (Some(ae), Some(be)) = (a_epoch, b_epoch) {
        match ae.cmp(&be) {
            std::cmp::Ordering::Less => return VersionComparison::LessThan,
            std::cmp::Ordering::Greater => return VersionComparison::GreaterThan,
            std::cmp::Ordering::Equal => {}
        }
    } else if a_epoch.is_some() {
        return VersionComparison::GreaterThan;
    } else if b_epoch.is_some() {
        return VersionComparison::LessThan;
    }

    let a_components = split_components(a_rest);
    let b_components = split_components(b_rest);

    debug!("比较组件: a={:?}, b={:?}", a_components, b_components);

    let min_len = a_components.len().min(b_components.len());
    for i in 0..min_len {
        let cmp = compare_component(&a_components[i], &b_components[i]);
        match cmp {
            std::cmp::Ordering::Less => return VersionComparison::LessThan,
            std::cmp::Ordering::Greater => return VersionComparison::GreaterThan,
            std::cmp::Ordering::Equal => {}
        }
    }

    if a_components.len() == b_components.len() {
        return VersionComparison::Equal;
    }

    if a_components.len() > b_components.len() {
        let extra = &a_components[b_components.len()..];
        if extra.iter().any(|c| c.starts_with('~') || is_prerelease_component(c)) {
            return VersionComparison::LessThan;
        }
        return VersionComparison::GreaterThan;
    }

    let extra = &b_components[a_components.len()..];
    if extra.iter().any(|c| c.starts_with('~') || is_prerelease_component(c)) {
        return VersionComparison::GreaterThan;
    }
    VersionComparison::LessThan
}

fn split_epoch(version: &str) -> (Option<u32>, &str) {
    if let Some(colon_idx) = version.find(':') {
        let epoch_str = &version[..colon_idx];
        if let Ok(epoch) = epoch_str.parse::<u32>() {
            return (Some(epoch), &version[colon_idx + 1..]);
        }
    }
    (None, version)
}

fn split_components(version: &str) -> Vec<String> {
    let mut components = Vec::new();
    let mut current = String::new();
    let mut prev_is_digit = false;
    let mut in_tilde = false;

    for c in version.chars() {
        if c == '~' {
            if !current.is_empty() {
                components.push(current);
            }
            current = String::new();
            current.push(c);
            in_tilde = true;
            prev_is_digit = false;
        } else if in_tilde {
            if c.is_ascii_alphanumeric() || c == '_' {
                current.push(c);
            } else {
                components.push(current);
                current = String::new();
                in_tilde = false;
                prev_is_digit = false;
            }
        } else if c == '_' {
            current.push(c);
            prev_is_digit = false;
        } else if c.is_ascii_digit() {
            if !prev_is_digit && !current.is_empty() && !current.ends_with('_') {
                components.push(current);
                current = String::new();
            }
            current.push(c);
            prev_is_digit = true;
        } else if c.is_ascii_alphabetic() {
            if prev_is_digit && !current.is_empty() && !current.ends_with('_') {
                components.push(current);
                current = String::new();
            }
            current.push(c);
            prev_is_digit = false;
        } else {
            if !current.is_empty() {
                components.push(current);
                current = String::new();
            }
            prev_is_digit = false;
        }
    }

    if !current.is_empty() {
        components.push(current);
    }

    components
}

fn compare_component(a: &str, b: &str) -> std::cmp::Ordering {
    let a_has_tilde = a.starts_with('~');
    let b_has_tilde = b.starts_with('~');

    if a_has_tilde && !b_has_tilde {
        return std::cmp::Ordering::Less;
    }
    if !a_has_tilde && b_has_tilde {
        return std::cmp::Ordering::Greater;
    }

    let a_clean = a.trim_start_matches('~');
    let b_clean = b.trim_start_matches('~');

    if a_clean.is_empty() && b_clean.is_empty() {
        return std::cmp::Ordering::Equal;
    }
    if a_clean.is_empty() {
        return std::cmp::Ordering::Less;
    }
    if b_clean.is_empty() {
        return std::cmp::Ordering::Greater;
    }

    let a_is_prerelease = is_prerelease_component(a_clean);
    let b_is_prerelease = is_prerelease_component(b_clean);

    if a_is_prerelease && !b_is_prerelease {
        return std::cmp::Ordering::Less;
    }
    if !a_is_prerelease && b_is_prerelease {
        return std::cmp::Ordering::Greater;
    }

    match (a_clean.parse::<i64>(), b_clean.parse::<i64>()) {
        (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a_clean.cmp(b_clean),
    }
}

fn is_prerelease_component(s: &str) -> bool {
    s.starts_with("alpha") || s.starts_with("beta") || s.starts_with("rc") || s.starts_with("pre") || s.starts_with("dev")
}

pub fn is_prerelease(version: &str) -> bool {
    let lower = version.to_lowercase();
    lower.contains("alpha") || lower.contains("beta") || lower.contains("rc") 
        || lower.contains("pre") || lower.contains("dev") || lower.contains("snapshot")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(compare_vercmp("1.2.3", "1.2.3"), VersionComparison::Equal);
        assert_eq!(compare_vercmp("2:1.2.3", "2:1.2.3"), VersionComparison::Equal);
        assert_eq!(compare_vercmp("1.2.3-1", "1.2.3-1"), VersionComparison::Equal);
    }

    #[test]
    fn test_simple_comparison() {
        assert_eq!(compare_vercmp("1.2.3", "1.2.4"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.4", "1.2.3"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.3.0", "1.2.9"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("2.0.0", "1.9.9"), VersionComparison::GreaterThan);
    }

    #[test]
    fn test_epoch() {
        assert_eq!(compare_vercmp("2:1.0.0", "1:1.0.0"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1:1.0.0", "2:1.0.0"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1:1.0.0", "1:1.0.0"), VersionComparison::Equal);
        assert_eq!(compare_vercmp("1:1.0.0", "1.0.0"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.0.0", "1:1.0.0"), VersionComparison::LessThan);
    }

    #[test]
    fn test_tilde() {
        assert_eq!(compare_vercmp("1.2.3~beta", "1.2.3"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.3", "1.2.3~beta"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3~alpha", "1.2.3~beta"), VersionComparison::LessThan);
    }

    #[test]
    fn test_alpha_numeric() {
        assert_eq!(compare_vercmp("1.2.3a", "1.2.3"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3", "1.2.3a"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.3a", "1.2.3b"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.3b", "1.2.3a"), VersionComparison::GreaterThan);
    }

    #[test]
    fn test_release_candidate() {
        assert_eq!(compare_vercmp("1.2.3rc1", "1.2.3"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.3", "1.2.3rc1"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3rc2", "1.2.3rc1"), VersionComparison::GreaterThan);
    }

    #[test]
    fn test_build_metadata() {
        assert_eq!(compare_vercmp("1.2.3+build1", "1.2.3"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3", "1.2.3+build1"), VersionComparison::LessThan);
    }

    #[test]
    fn test_underscore() {
        assert_eq!(compare_vercmp("1.2.3_beta", "1.2.3"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3_beta1", "1.2.3_beta2"), VersionComparison::LessThan);
    }

    #[test]
    fn test_different_lengths() {
        assert_eq!(compare_vercmp("1.2", "1.2.3"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("1.2.3", "1.2"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.0", "1.2"), VersionComparison::GreaterThan);
    }

    #[test]
    fn test_complex() {
        assert_eq!(compare_vercmp("2:1.2.3~alpha-1", "2:1.2.3"), VersionComparison::LessThan);
        assert_eq!(compare_vercmp("2:1.2.3_beta", "1:1.2.3"), VersionComparison::GreaterThan);
        assert_eq!(compare_vercmp("1.2.3-1", "1.2.3-2"), VersionComparison::LessThan);
    }
}