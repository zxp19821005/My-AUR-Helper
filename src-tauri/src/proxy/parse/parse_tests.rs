/**
 * parse_tests.rs - 代理解析模块单元测试
 *
 * 覆盖从实际代理规则文件解析代理数量、注释条目过滤，
 * 以及 `normalize_proxy_url` 的目标协议头约定推断。
 */
use super::*;

#[test]
fn test_parse_proxy_counts_from_file() {
    // 读取实际的代理规则文件
    let content = std::fs::read_to_string(
        "/home/zxp-archlinux/.config/com.zxp19821005.aur-helper/tmp/proxy_rules.js"
    ).expect("无法读取代理规则文件");

    let proxies = parse_js_content(&content).expect("解析失败");

    // 按类型统计
    let download: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Download).collect();
    let clone: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Clone).collect();
    let raw: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Raw).collect();
    let ssh: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Ssh).collect();

    println!("总代理数: {}", proxies.len());
    println!("  下载代理: {} (download)", download.len());
    println!("  克隆代理: {} (clone)", clone.len());
    println!("  RAW代理: {} (raw)", raw.len());
    println!("  SSH代理: {} (ssh)", ssh.len());

    // 验证预期数量
    assert_eq!(download.len(), 30, "下载代理应为 30 个");
    assert_eq!(clone.len(), 5, "克隆代理应为 5 个");
    assert_eq!(raw.len(), 8, "RAW 代理应为 8 个");
    assert_eq!(ssh.len(), 1, "SSH 代理应为 1 个");
    assert_eq!(proxies.len(), 44, "总代理数应为 44");
}

#[test]
fn test_comment_entries_are_not_parsed() {
    let content = std::fs::read_to_string(
        "/home/zxp-archlinux/.config/com.zxp19821005.aur-helper/tmp/proxy_rules.js"
    ).expect("无法读取代理规则文件");

    let proxies = parse_js_content(&content).expect("解析失败");

    // 验证被注释掉的条目确实没有被导入
    let commented_urls = vec![
        "https://gh.api.99988866.xyz/https://github.com",
        "https://hub.glowp.xyz/https://github.com",
        "https://gitdl.cn/https://github.com",
        "https://gitproxy.click/https://github.com",
        "https://cdn.moran233.xyz/https://github.com",
    ];

    for commented_url in commented_urls {
        assert!(
            !proxies.iter().any(|p| p.url == commented_url),
            "被注释掉的 URL 不应被导入: {}",
            commented_url
        );
    }
}

#[test]
fn normalize_infers_strip_convention_from_script_entry() {
    // isteed 类：原始条目裸主机后缀 → 源站 + 去协议头约定
    assert_eq!(
        normalize_proxy_url("https://cors.isteed.cc/github.com"),
        ("https://cors.isteed.cc".to_string(), true)
    );
    // crashmc 类：原始条目带协议头后缀 → 源站 + 保留协议头约定
    assert_eq!(
        normalize_proxy_url("https://cdn.crashmc.com/https://github.com"),
        ("https://cdn.crashmc.com".to_string(), false)
    );
    // down.npee 类：?https:// 查询形式后缀 → 源站 + 保留协议头约定
    assert_eq!(
        normalize_proxy_url("https://down.npee.cn/?https://github.com"),
        ("https://down.npee.cn".to_string(), false)
    );
    // 干净源站（仅末尾斜杠）→ 源站 + 默认保留协议头
    assert_eq!(
        normalize_proxy_url("https://cdn.crashmc.com/"),
        ("https://cdn.crashmc.com".to_string(), false)
    );
}
