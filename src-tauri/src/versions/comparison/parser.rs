/**
 * parser.rs - 版本字符串解析器
 *
 * 功能：
 * - 解析 epoch 前缀 (如 2:1.0.0)
 * - 拆分版本号为比较组件
 * - 判断组件是否为预发布标记
 * - 比较单个组件的大小
 */

/// 分离 epoch 前缀和版本号主体
///
/// # 参数
/// - `version`: 完整的版本号字符串
///
/// # 返回
/// - `(Some(epoch), rest)`: 存在 epoch 前缀
/// - `(None, version)`: 不存在 epoch 前缀
pub fn split_epoch(version: &str) -> (Option<u32>, &str) {
    if let Some(colon_idx) = version.find(':') {
        let epoch_str = &version[..colon_idx];
        if let Ok(epoch) = epoch_str.parse::<u32>() {
            return (Some(epoch), &version[colon_idx + 1..]);
        }
    }
    (None, version)
}

/// 将版本号拆分为可比较的组件列表
///
/// 拆分规则：
/// - 数字和字母之间会拆分
/// - tilde (~) 开头的组件单独提取
/// - 下划线 (_) 作为分隔符保留
pub fn split_components(version: &str) -> Vec<String> {
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

/// 判断组件是否为预发布标记
pub fn is_prerelease_component(s: &str) -> bool {
    s.starts_with("alpha")
        || s.starts_with("beta")
        || s.starts_with("rc")
        || s.starts_with("pre")
        || s.starts_with("dev")
}

/// 比较两个版本组件的大小
///
/// 比较规则：
/// 1. tilde (~) 开头的组件最小（预发布）
/// 2. 预发布标记（alpha/beta/rc 等）小于普通组件
/// 3. 纯数字按数值比较
/// 4. 数字小于字母
/// 5. 字母按字典序比较
pub fn compare_component(a: &str, b: &str) -> std::cmp::Ordering {
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
