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
 */
#[cfg(test)]
mod tests {
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
}
