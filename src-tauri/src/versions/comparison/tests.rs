/**
 * tests.rs - 版本比较算法的单元测试
 *
 * 测试覆盖：
 * - 相等版本比较
 * - 简单版本比较
 * - epoch 前缀处理
 * - tilde 预发布标记
 * - 字母后缀
 * - rc/dev/pre 预发布版本
 * - 构建元数据
 * - 下划线分隔符
 * - 不同长度的版本号
 * - 复杂组合场景
 *
 * 注意：本文件由 comparison.rs 的 `#[cfg(test)] mod tests;` 声明，顶层即模块体。
 */
use crate::versions::comparison::{compare_versions as compare_vercmp, VersionComparison};

#[test]
fn test_equal() {
    assert_eq!(compare_vercmp("1.2.3", "1.2.3"), VersionComparison::Equal);
    assert_eq!(
        compare_vercmp("2:1.2.3", "2:1.2.3"),
        VersionComparison::Equal
    );
    assert_eq!(
        compare_vercmp("1.2.3-1", "1.2.3-1"),
        VersionComparison::Equal
    );
}

#[test]
fn test_simple_comparison() {
    assert_eq!(
        compare_vercmp("1.2.3", "1.2.4"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.4", "1.2.3"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.3.0", "1.2.9"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("2.0.0", "1.9.9"),
        VersionComparison::GreaterThan
    );
}

#[test]
fn test_epoch() {
    assert_eq!(
        compare_vercmp("2:1.0.0", "1:1.0.0"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1:1.0.0", "2:1.0.0"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1:1.0.0", "1:1.0.0"),
        VersionComparison::Equal
    );
    assert_eq!(
        compare_vercmp("1:1.0.0", "1.0.0"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.0.0", "1:1.0.0"),
        VersionComparison::LessThan
    );
}

#[test]
fn test_tilde() {
    assert_eq!(
        compare_vercmp("1.2.3~beta", "1.2.3"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.3", "1.2.3~beta"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3~alpha", "1.2.3~beta"),
        VersionComparison::LessThan
    );
}

#[test]
fn test_alpha_numeric() {
    assert_eq!(
        compare_vercmp("1.2.3a", "1.2.3"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3", "1.2.3a"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.3a", "1.2.3b"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.3b", "1.2.3a"),
        VersionComparison::GreaterThan
    );
}

#[test]
fn test_release_candidate() {
    assert_eq!(
        compare_vercmp("1.2.3rc1", "1.2.3"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.3", "1.2.3rc1"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3rc2", "1.2.3rc1"),
        VersionComparison::GreaterThan
    );
}

#[test]
fn test_build_metadata() {
    assert_eq!(
        compare_vercmp("1.2.3+build1", "1.2.3"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3", "1.2.3+build1"),
        VersionComparison::LessThan
    );
}

#[test]
fn test_underscore() {
    assert_eq!(
        compare_vercmp("1.2.3_beta", "1.2.3"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3_beta1", "1.2.3_beta2"),
        VersionComparison::LessThan
    );
}

#[test]
fn test_different_lengths() {
    assert_eq!(compare_vercmp("1.2", "1.2.3"), VersionComparison::LessThan);
    assert_eq!(
        compare_vercmp("1.2.3", "1.2"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.0", "1.2"),
        VersionComparison::GreaterThan
    );
}

#[test]
fn test_complex() {
    assert_eq!(
        compare_vercmp("2:1.2.3~alpha-1", "2:1.2.3"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("2:1.2.3_beta", "1:1.2.3"),
        VersionComparison::GreaterThan
    );
    assert_eq!(
        compare_vercmp("1.2.3-1", "1.2.3-2"),
        VersionComparison::LessThan
    );
}

/// pkgrel 不应被当成版本组件错位比较：pkgver 不等时以 pkgver 为准
#[test]
fn test_pkgrel_not_positional() {
    // 缓存 9.0.1-1 比备份 9.0-5 更新（pkgver 9.0.1 > 9.0）
    assert_eq!(
        compare_vercmp("2:9.0.1-1", "2:9.0-5"),
        VersionComparison::GreaterThan
    );
    // 反向：上游 9.0.1 比 AUR 9.0-5 更新
    assert_eq!(
        compare_vercmp("2:9.0-5", "2:9.0.1"),
        VersionComparison::LessThan
    );
    // pkgver 段数不同：1.2 < 1.2.3，pkgrel 不应干扰（修复 1.2-5 vs 1.2.3-1 误判）
    assert_eq!(
        compare_vercmp("1.2-5", "1.2.3-1"),
        VersionComparison::LessThan
    );
    // pkgver 相等时才比较 pkgrel
    assert_eq!(
        compare_vercmp("1.2.3-1", "1.2.3-5"),
        VersionComparison::LessThan
    );
    assert_eq!(
        compare_vercmp("1.2.3-5", "1.2.3-1"),
        VersionComparison::GreaterThan
    );
    // pkgrel 缺失视为 0
    assert_eq!(
        compare_vercmp("1.2.3-1", "1.2.3"),
        VersionComparison::GreaterThan
    );
}

// ==================== 与系统 vercmp 的一致性对照 ====================

/// 调用系统 `vercmp` 取得权威判定
///
/// # 参数
/// - `a`, `b`: 待比较的两个版本串
///
/// # 返回
/// - `Some(VersionComparison)`: 系统命令可用且输出可解析
/// - `None`: 非 Arch 环境（无 vercmp 命令）或输出异常，调用方应跳过断言
fn system_vercmp(a: &str, b: &str) -> Option<VersionComparison> {
    let output = std::process::Command::new("vercmp")
        .args([a, b])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "-1" => Some(VersionComparison::LessThan),
        "0" => Some(VersionComparison::Equal),
        "1" => Some(VersionComparison::GreaterThan),
        _ => None,
    }
}

#[test]
fn matches_system_vercmp_on_shared_semantics() {
    // 期望值来自本机 pacman 7.1.0 的 vercmp 实测输出，逐条以系统命令复核。
    // 自实现是「ALPM 风格 + 项目自定义预发布/tilde 扩展」，与系统 vercmp 并非
    // 逐位等价，以下两组为刻意差异，不纳入一致性断言：
    //   - `1.0a`     vs `1.0`：系统 -1，自实现 GreaterThan（额外组件默认更新）
    //   - `1.0~beta` vs `1.0`：系统  1，自实现 LessThan（tilde 视为预发布）
    // 若把它们纳入，等于把项目既有语义错误地改成 pacman 语义。
    let cases: &[(&str, &str, VersionComparison)] = &[
        ("1.2.3", "1.2.4", VersionComparison::LessThan),
        ("1.2.10", "1.2.9", VersionComparison::GreaterThan),
        ("1.0", "1.0.0", VersionComparison::LessThan),
        ("1.0b", "1.0a", VersionComparison::GreaterThan),
        ("1.0rc1", "1.0", VersionComparison::LessThan),
        ("1.0-1", "1.0-2", VersionComparison::LessThan),
        ("2:1.0", "1:9.9", VersionComparison::GreaterThan),
        ("9.0.1-1", "9.0-5", VersionComparison::GreaterThan),
        ("1.0", "1.0", VersionComparison::Equal),
        ("1.0+build1", "1.0", VersionComparison::GreaterThan),
        (
            "2.0.1.r0.g30a6260",
            "2.0.1.r5.g9a27946",
            VersionComparison::LessThan,
        ),
    ];

    let mut checked = 0usize;
    for &(a, b, expected) in cases {
        if let Some(system) = system_vercmp(a, b) {
            assert_eq!(system, expected, "系统 vercmp 基准值变化: {} vs {}", a, b);
            assert_eq!(
                compare_vercmp(a, b),
                expected,
                "自实现偏离 vercmp: {} vs {}",
                a,
                b
            );
            checked += 1;
        }
    }
    if checked == 0 {
        eprintln!("跳过一致性对照：当前环境无系统 vercmp 命令（非 Arch Linux）");
    }
}
