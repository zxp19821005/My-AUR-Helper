/// 去除常见的文件扩展名与架构/平台后缀
///
/// 处理三类后缀：
/// 1. 文件扩展名（.deb / .rpm / .AppImage ...）——按结尾匹配去除。
/// 2. 点分隔的架构后缀（.x86_64 / .amd64 / .aarch64 ...）——按结尾匹配去除。
/// 3. 连字符分隔的架构/平台/发行版后缀（如 `-amd64-web`、`-uos-amd64`、`-linux`）。
///    从右向左连续剥除命中已知 token 的尾段（amd64/uos/web/linux ...），
///    仅保留 X.Y.Z 主版本。
///
/// 关键约束：prerelease（-alpha/-beta/-rc）与 pkgrel（-N）不在 token 集合内，
/// 不会被误删，保证语义版本完整（详见本模块单测）。
///
/// @param version - 包含可能扩展名/后缀的版本字符串
/// @returns 清洗后的版本字符串
pub(crate) fn strip_file_extensions(version: &str) -> String {
    let extensions = [
        ".AppImage",
        ".appimage",
        ".flatpak",
        ".deb",
        ".rpm",
        ".exe",
        ".msi",
        ".dmg",
        ".pkg",
        ".tar.gz",
        ".tar.xz",
        ".zip",
        ".tar.bz2",
        ".tar.zst",
        ".7z",
        ".snap",
        ".AppImage.zsync",
    ];

    let mut result = version.to_string();
    for ext in &extensions {
        if result.ends_with(ext) {
            result = result[..result.len() - ext.len()].to_string();
            break;
        }
    }

    // 去除点分隔的架构后缀 (如 .x86_64, .aarch64, .arm64, .amd64, .i386 等)
    let dot_arch = [
        ".x86_64",
        ".aarch64",
        ".arm64",
        ".amd64",
        ".i386",
        ".i686",
        ".armv7l",
        ".armhf",
        ".noarch",
        ".all",
        ".universal",
    ];
    for arch in &dot_arch {
        if result.ends_with(arch) {
            result = result[..result.len() - arch.len()].to_string();
            break;
        }
    }

    // 去除连字符分隔的架构/平台/发行版后缀（如 -amd64-web、-uos-amd64、-linux）。
    // 从右向左连续剥离命中已知 token 的尾段；prerelease（-alpha 等）与
    // pkgrel（-N）不属于 token，不会被误删，保证语义版本完整。
    let hyphen_tokens = [
        "amd64",
        "x86_64",
        "aarch64",
        "arm64",
        "i386",
        "i686",
        "armv7l",
        "armhf",
        "noarch",
        "all",
        "universal",
        "x64",
        "web",
        "linux",
        "win",
        "windows",
        "mac",
        "macos",
        "darwin",
        "portable",
        "setup",
        "installer",
        "uos",
        "debian",
        "ubuntu",
        "kylin",
        "centos",
        "fedora",
        "opensuse",
        "suse",
        "rhel",
    ];
    let segments: Vec<&str> = result.split('-').collect();
    if segments.len() > 1 {
        let mut end = segments.len();
        // 从最后一个段向版本核心回扫，连续剥除命中 token 的尾段；
        // 一旦遇到非 token 段（如 prerelease / pkgrel）立即停止，避免误删。
        while end > 1 {
            let seg = segments[end - 1];
            let base = seg.split(['.', '+']).next().unwrap_or(seg);
            if hyphen_tokens.contains(&base) {
                end -= 1;
            } else {
                break;
            }
        }
        if end < segments.len() {
            result = segments[..end].join("-");
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归：ynote-desktop-bin 的 Content-Disposition 为
    /// `有道云笔记-8.2.92-amd64-web.deb`，旧逻辑把 `-amd64-web` 当成版本前缀，
    /// 提取出 `8.2.92-amd64-web`（误报需更新）。修复后应只保留 X.Y.Z 主版本。
    #[test]
    fn test_strip_arch_flavor_hyphen() {
        assert_eq!(strip_file_extensions("8.2.92-amd64-web.deb"), "8.2.92");
        assert_eq!(strip_file_extensions("8.2.92-amd64.deb"), "8.2.92");
        assert_eq!(strip_file_extensions("8.2.92-web.deb"), "8.2.92");
        assert_eq!(strip_file_extensions("8.2.92-linux.deb"), "8.2.92");
        assert_eq!(strip_file_extensions("8.2.92-win.exe"), "8.2.92");
        assert_eq!(strip_file_extensions("8.2.92-x64.deb"), "8.2.92");
    }

    /// 回归：weimail-bin 的 Content-Disposition 为
    /// `weimail-2.3.7.0-uos-amd64.deb`，旧逻辑只剥掉了末尾 `-amd64`，
    /// 残留 `-uos` 得到 `2.3.7.0-uos`。修复后从右向左连续剥除发行版/架构 token，
    /// 应为 `2.3.7.0`。
    #[test]
    fn test_strip_distro_then_arch() {
        assert_eq!(strip_file_extensions("2.3.7.0-uos-amd64.deb"), "2.3.7.0");
        assert_eq!(strip_file_extensions("2.3.7.0-uos.deb"), "2.3.7.0");
        assert_eq!(strip_file_extensions("1.2.3-debian-amd64.deb"), "1.2.3");
        assert_eq!(strip_file_extensions("1.2.3-kylin-x64.deb"), "1.2.3");
    }

    /// 兼容旧的「点分隔」架构后缀写法（如 .x86_64 / .aarch64）
    #[test]
    fn test_strip_arch_dot() {
        assert_eq!(strip_file_extensions("1.2.3.x86_64.rpm"), "1.2.3");
        assert_eq!(strip_file_extensions("1.2.3.amd64.deb"), "1.2.3");
        assert_eq!(strip_file_extensions("1.2.3-aarch64.AppImage"), "1.2.3");
    }

    /// 不能误伤 prerelease 与 pkgrel：它们不是架构/平台 token
    #[test]
    fn test_strip_preserves_prerelease_and_pkgrel() {
        assert_eq!(
            strip_file_extensions("1.0.0-alpha.16.deb"),
            "1.0.0-alpha.16"
        );
        assert_eq!(strip_file_extensions("1.0.0-beta.2.deb"), "1.0.0-beta.2");
        assert_eq!(strip_file_extensions("2.0.0-rc.1.deb"), "2.0.0-rc.1");
        assert_eq!(strip_file_extensions("8.2.92-1.deb"), "8.2.92-1");
    }
}
