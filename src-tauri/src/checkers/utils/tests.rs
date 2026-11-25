// @LINE_LIMIT_EXEMPT: 测试文件
/**
 * checkers/utils/tests.rs - 版本提取与清洗工具单元测试
 *
 * 功能：集中承载原 utils.rs 的正则提取、clean_version、
 * normalize_extracted_version 等行为回归测试。
 *
 * 工作流程：通过 `use super::*` 复用 utils barrel re-export 的公开函数，
 * 用例线性堆叠，验证 prerelease 保留、app 名前缀/后缀剥离等真实 bug 修复。
 *
 * 适用场景：`cargo test --lib` 回归检查器版本提取语义不被重构改变。
 */
use super::*;

/// clean_version 必须保留 prerelease/build 元数据，否则 GitHub GraphQL 批量
/// 检查会把 v1.0.0-alpha.16 错误地截断为 1.0.0，导致版本比较失真。
/// 真实 bug 复现：deskaide/deskaide 仓库全部 tag 都是 v1.0.0-alpha.X，
/// 旧正则 `[^0-9]*?(v?\d[\d.]*)` 把它们全部截为 1.0.0，max_by_vercmp 比较
/// 全部返回 Equal，最终返回 1.0.0（错的），用户 AUR 1.0.0_alpha.16 被认为无需更新。
/// 正则提取：无捕获组时返回整段匹配；有捕获组时返回首个捕获组。
/// 实证：helio-workstation-bin 用户配置 `helio-\d\.\d+-x64.deb` 无捕获组，
/// 因此拿到整段 `helio-3.18-x64.deb`（含 app 名前缀与 .deb 后缀）。
/// 正确配置应为 `helio-(\d\.\d+)-x64\.deb`，用捕获组取 `3.18`。
#[test]
fn test_regex_extraction_capture_group_semantics() {
    // 无捕获组 → 整段匹配
    assert_eq!(
        extract_version_with_regex("helio-3.18-x64.deb", r"helio-\d\.\d+-x64\.deb"),
        Some("helio-3.18-x64.deb".to_string())
    );
    // 有捕获组 → 首位捕获组
    assert_eq!(
        extract_version_with_regex("helio-3.18-x64.deb", r"helio-(\d\.\d+)-x64\.deb"),
        Some("3.18".to_string())
    );
}

#[test]
fn test_clean_version_preserves_prerelease() {
    assert_eq!(clean_version("v1.0.0-alpha.16"), "1.0.0-alpha.16");
    assert_eq!(clean_version("1.0.0-alpha.16"), "1.0.0-alpha.16");
    assert_eq!(clean_version("v1.0.0-beta.2"), "1.0.0-beta.2");
    assert_eq!(clean_version("v2.0.0-rc.1"), "2.0.0-rc.1");
}

#[test]
fn test_clean_version_strips_v_prefix() {
    assert_eq!(clean_version("v1.0.0"), "1.0.0");
    assert_eq!(clean_version("V1.0.0"), "1.0.0");
    assert_eq!(clean_version("v1.2.3"), "1.2.3");
}

#[test]
fn test_clean_version_appname_prefix() {
    assert_eq!(clean_version("appname-v1.0.0"), "1.0.0");
    assert_eq!(clean_version("my-app-v1.2.3"), "1.2.3");
}

#[test]
fn test_clean_version_keeps_pkgrel() {
    assert_eq!(clean_version("v1.0.0-1"), "1.0.0-1");
}

/// normalize_extracted_version 必须兜底「无捕获组正则」返回整段匹配的情况：
/// `helio-\d\.\d+-x64.deb` 匹配到 `helio-3.18-x64.deb`，归一化后应剥离
/// `.deb`/`x64` 后缀与 `helio-` 前缀，得到 `3.18`。这正是 helio-workstation-bin
/// 误报的复现与修复验证。
#[test]
fn test_normalize_strips_app_prefix_and_suffix() {
    assert_eq!(normalize_extracted_version("helio-3.18-x64.deb"), "3.18");
    // 对已是干净版本号的输入幂等
    assert_eq!(normalize_extracted_version("3.18"), "3.18");
    assert_eq!(
        normalize_extracted_version("1.0.0-alpha.16"),
        "1.0.0-alpha.16"
    );
    // 与 redirect/url 路径一致：note 类带 -amd64-web.deb 后缀
    assert_eq!(
        normalize_extracted_version("有道云笔记-8.2.92-amd64-web.deb"),
        "8.2.92"
    );
}

/// normalize_extracted_version 必须兜底「无捕获组正则匹配到整段带 app 名与构建号」
/// 的情况：moneydance-bin 的 `Moneydance\ \d.+\d+` 匹配到 `Moneydance 2024.4 (5253`，
/// 清洗后残留噪声，应回退提取首个语义版本号 `2024.4`。
#[test]
fn test_normalize_extracts_embedded_version() {
    assert_eq!(
        normalize_extracted_version("Moneydance 2024.4 (5253"),
        "2024.4"
    );
    // 闭合括号也不影响（只取版本子串）
    assert_eq!(
        normalize_extracted_version("Moneydance 2024.4 (5253)"),
        "2024.4"
    );
    // 已是纯版本号则不变
    assert_eq!(normalize_extracted_version("3.18"), "3.18");
    assert_eq!(
        normalize_extracted_version("1.0.0-alpha.16"),
        "1.0.0-alpha.16"
    );
    // 退化情形：无点分版本，clean_version 仅剥掉 `build ` 前缀，无版本子串可兜底
    assert_eq!(normalize_extracted_version("build 5253"), "5253");
}

#[test]
fn test_clean_version_passthrough_non_version() {
    // 看起来不像版本号的字符串（无数字）原样保留
    assert_eq!(clean_version("continuous"), "continuous");
    assert_eq!(clean_version("latest"), "latest");
}
