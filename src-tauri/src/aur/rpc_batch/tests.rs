use super::*;

// ==================== encode_info_body ====================

#[test]
fn test_encode_info_body_plain_exact() {
    let names = vec!["yay".to_string(), "paru".to_string()];
    let body = encode_info_body(&names);
    // `[` 和 `]` 必须被百分号编码
    assert_eq!(body, "arg%5B%5D=yay&arg%5B%5D=paru");
}

#[test]
fn test_encode_info_body_plus_is_escaped() {
    let names = vec!["lib32-libsigc++".to_string()];
    let body = encode_info_body(&names);
    // 字面 `+` 必须转义为 %2B，否则服务端会按 form 规则解码成空格
    assert!(body.contains("%2B"), "body 应含 %2B，实际: {body}");
    // 绝不能出现裸 +
    assert!(!body.contains('+'), "body 不应含裸 +，实际: {body}");
    // round-trip：解回来必须与原始包名完全一致
    let decoded: Vec<(String, String)> = url::form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].0, "arg[]");
    assert_eq!(decoded[0].1, "lib32-libsigc++");
}

#[test]
fn test_encode_info_body_special_chars_roundtrip() {
    let names = vec![
        "has space".to_string(),
        "中文包名".to_string(),
        "a&b".to_string(),
        "c#d".to_string(),
    ];
    let body = encode_info_body(&names);
    // 空格会被 form 编码为 `+`，body 中不应出现裸空格
    assert!(!body.contains(' '), "body 不应含裸空格，实际: {body}");
    let decoded: Vec<(String, String)> = url::form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();
    assert_eq!(decoded.len(), names.len());
    for (i, (key, val)) in decoded.iter().enumerate() {
        assert_eq!(key, "arg[]", "第 {i} 个 key 应恒为 arg[]");
        assert_eq!(val, &names[i], "第 {i} 个包 round-trip 不一致");
    }
}

#[test]
fn test_encode_info_body_empty() {
    let names: Vec<String> = Vec::new();
    assert_eq!(encode_info_body(&names), "");
}

// ==================== looks_like_intercepted ====================

#[test]
fn test_looks_like_intercepted_json_ok() {
    assert!(!looks_like_intercepted(
        "application/json",
        "{\"resultcount\":0}"
    ));
}

#[test]
fn test_looks_like_intercepted_anubis_html() {
    let ct = "text/html; charset=utf-8";
    let body = "<!doctype html><html><head><title>Oh noes!</title></head></html>";
    assert!(looks_like_intercepted(ct, body));
}

#[test]
fn test_looks_like_intercepted_missing_ct_empty_body() {
    // content-type 缺失 + 空响应体，应保守判定为异常
    assert!(looks_like_intercepted("", ""));
}

#[test]
fn test_looks_like_intercepted_leading_whitespace_html() {
    // 前置空白后仍以 `<` 开头，应判为拦截
    assert!(looks_like_intercepted("text/plain", "   \n\t< html"));
}

#[test]
fn test_looks_like_intercepted_case_insensitive_ct() {
    // content-type 大小写混写仍应识别为 JSON
    assert!(!looks_like_intercepted("Application/JSON", "{}"));
}

// ==================== preview ====================

#[test]
fn test_preview_truncates_by_chars() {
    let s = "中文测试🎉emoji";
    let p = preview(s, 4);
    assert_eq!(p, "中文测试");
    // 截断长度按字符而非字节计算
    assert_eq!(p.chars().count(), 4);
    // 截断到 emoji 边界也不得 panic，且按字符计数
    let p2 = preview(s, 5);
    assert_eq!(p2, "中文测试🎉");
    assert_eq!(p2.chars().count(), 5);
}

#[test]
fn test_preview_longer_than_text_and_empty() {
    let s = "短";
    assert_eq!(preview(s, 100), "短");
    assert_eq!(preview("", 10), "");
    assert_eq!(preview(s, 0), "");
}

// ==================== get_packages_info 批次大小兜底 ====================

/// `batch_size=0` 必须被兜底为 1：修复前 `slice::chunks(0)` 会直接 panic，
/// 而该值可由用户在设置里把 `aur_batch_size` 填成 0 触发（`parse_u32("0", 50) == 0`）。
///
/// 手法：client 强制把 AUR 域名解析到不可达的 127.0.0.1:1，一旦真的进入请求会立刻
/// 连接失败。该函数对批次失败的设计是「记 warn 后继续、最终返回已拿到的结果」，
/// 所以修复后的预期是 `Ok(空 vec)`；而修复前会在分块时直接 panic。
#[tokio::test]
async fn test_get_packages_info_batch_size_zero_does_not_panic() {
    let unreachable = std::net::SocketAddr::from(([127, 0, 0, 1], 1));
    let client = Client::builder()
        .resolve("aur.archlinux.org", unreachable)
        .build()
        .expect("构造探测用 client 失败");

    let pkgnames = vec!["foo-bin".to_string()];
    let result = get_packages_info(&client, &pkgnames, 0, 0).await;
    assert!(result.is_ok(), "batch_size=0 应被兜底为 1，而不是 panic");
    assert!(result.unwrap().is_empty(), "不可达网络下不应拿到任何结果");
}
