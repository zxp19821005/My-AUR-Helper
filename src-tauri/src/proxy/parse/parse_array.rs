//! parse_array.rs - 代理 JS 数组提取
//!
//! 提供从代理规则 JS 文件中精确提取指定数组字符串元素的纯解析逻辑。
//! 采用字节级状态机，正确处理字符串、转义与注释内的括号。

/// 从 JavaScript 内容中提取指定数组的所有字符串元素
/// 使用状态机精确处理字符串和注释中的括号
/// @param content - JavaScript 文件内容
/// @param array_name - 要提取的数组变量名（如 "download_url_us"）
/// @returns 提取到的字符串列表
pub(crate) fn extract_array(content: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;

    // 1. 找到数组声明起始位置：array_name = [
    let pattern = format!(r"{}\s*=\s*\[", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let m = re.find(content)?;

    // 2. 状态机：从 '[' 开始精确跟踪括号匹配
    //    跟踪状态：字符串内、注释内、括号深度
    let bytes = content.as_bytes();
    let start = m.start() + m.as_str().len() - 1; // '[' 的位置
    let mut i = start + 1;
    let mut depth = 1;
    let mut in_string = false;
    let mut string_quote: u8 = 0;

    while i < bytes.len() && depth > 0 {
        let ch = bytes[i];

        // 处理注释（// 到行尾，仅当不在字符串中）
        if !in_string && ch == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // 处理字符串
        if ch == b'\'' || ch == b'"' {
            if in_string && ch == string_quote {
                in_string = false;
            } else if !in_string {
                in_string = true;
                string_quote = ch;
            }
            i += 1;
            continue;
        }

        // 处理转义字符（在字符串中）
        if in_string && ch == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }

        // 统计括号（仅在字符串和注释之外）
        if !in_string {
            if ch == b'[' {
                depth += 1;
            } else if ch == b']' {
                depth -= 1;
            }
        }

        if depth > 0 {
            i += 1;
        }
    }

    if depth != 0 {
        return None; // 括号不匹配
    }

    let array_content = &content[start + 1..i];

    // 3. 解析数组内容中的字符串字面量
    let mut items = Vec::new();
    let bytes = array_content.as_bytes();
    let len = bytes.len();
    let mut pos = 0;

    while pos < len {
        // 跳过空白字符
        if bytes[pos] == b' ' || bytes[pos] == b'\n' || bytes[pos] == b'\r' || bytes[pos] == b'\t' {
            pos += 1;
            continue;
        }

        // 跳过注释行
        if bytes[pos] == b'/' && pos + 1 < len && bytes[pos + 1] == b'/' {
            while pos < len && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        // 找到字符串的开始
        if bytes[pos] == b'\'' || bytes[pos] == b'"' {
            let quote = bytes[pos];
            let s = pos + 1;
            pos += 1;

            // 找到字符串的结束（处理转义）
            while pos < len && bytes[pos] != quote {
                if bytes[pos] == b'\\' && pos + 1 < len {
                    pos += 2;
                } else {
                    pos += 1;
                }
            }

            if pos < len {
                let item = String::from_utf8_lossy(&bytes[s..pos]).to_string();
                items.push(item);
                pos += 1;
            }
        } else {
            pos += 1;
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}
