/**
 * sudoers.rs - sudoers 规则追加命令与免密探测公共封装
 *
 * 功能：
 * - build_sudoers_append_command: 生成幂等追加一条规则的提权命令
 * - is_rule_effective / are_all_rules_effective / is_any_rule_effective: 免密探测
 *
 * 背景：
 * /etc/sudoers.d/aur-helper-backup 由「备份安装 / 缓存安装 / 缓存清理」三个入口共用，
 * 三处原本各自拼同一条 `sudo sh -c 'grep -qF … || printf … chmod 440'` 命令，
 * 且各自重复 `sudo -n -l` + NOPASSWD 匹配的组合逻辑，共 3 + 5 处重复。
 * 本模块把模板与探测收敛为公共方法，底层仍复用 backup_install_helpers 中唯一的
 * sudo_privileges_text / has_nopasswd_rule 实现，不新增第二份子进程调用。
 *
 * 安全约束：生成的命令必须逐字等价于抽取前模板——规则不含单引号才可包进 sh -c，
 * 一律用 `>>` 追加而非覆盖，避免冲掉其它入口已配置的规则。
 */
use super::backup_install_helpers::{has_nopasswd_rule, sudo_privileges_text};

/// 三个安装/清理入口共用的 sudoers 片段文件
const SUDOERS_FILE: &str = "/etc/sudoers.d/aur-helper-backup";

/// 生成幂等追加一条 sudoers 规则的提权命令。
///
/// 输出与抽取前模板逐字一致：单次 `sudo sh -c` 内完成
/// `grep -qF` 幂等判断、`printf` 追加写入、`chmod 440` 权限修正。
///
/// # 参数
/// - `rule`: 用于 `grep -qF` 幂等判断的规则文本（同时是 line 的 NOPASSWD 命令部分）
/// - `line`: 实际写入文件的完整 sudoers 行（含用户名与 NOPASSWD 前缀）
///
/// # 返回
/// 可直接交给前端展示/执行的 shell 命令字符串
pub(crate) fn build_sudoers_append_command(rule: &str, line: &str) -> String {
    format!(
        "sudo sh -c 'grep -qF \"{rule}\" {file} || printf \"%s\\n\" \"{line}\" >> {file}; chmod 440 {file}'",
        rule = rule,
        line = line,
        file = SUDOERS_FILE,
    )
}

/// 免交互判断单条 sudoers 规则当前是否已对当前用户生效。
///
/// 读取 `sudo -n -l` 实际生效的 NOPASSWD 规则；读取失败一律返回 false，
/// 与抽取前 `Some(text) => has_nopasswd_rule(..), None => false` 等价。
///
/// # 参数
/// - `rule`: 待检测的规则命令片段
///
/// # 返回
/// `true` 表示规则已生效（无需重复配置）
pub(crate) async fn is_rule_effective(rule: &str) -> bool {
    match sudo_privileges_text().await {
        Some(text) => has_nopasswd_rule(&text, rule),
        None => false,
    }
}

/// 免交互判断一组 sudoers 规则是否全部生效。
///
/// 供「缓存安装」对每个启用缓存目录逐一校验时使用；
/// 读取失败返回 false（不误判为已配置）。
///
/// # 参数
/// - `rules`: 待检测的规则命令片段列表
///
/// # 返回
/// `true` 表示所有规则均已生效
pub(crate) async fn are_all_rules_effective(rules: &[String]) -> bool {
    match sudo_privileges_text().await {
        Some(text) => rules.iter().all(|r| has_nopasswd_rule(&text, r)),
        None => false,
    }
}

/// 免交互判断一组 sudoers 规则中是否至少一条生效。
///
/// 供「检测缓存安装免密配置」使用：任一缓存目录命中即视为可用；
/// 读取失败返回 false。
///
/// # 参数
/// - `rules`: 待检测的规则命令片段列表
///
/// # 返回
/// `true` 表示至少一条规则已生效
pub(crate) async fn is_any_rule_effective(rules: &[String]) -> bool {
    match sudo_privileges_text().await {
        Some(text) => rules.iter().any(|r| has_nopasswd_rule(&text, r)),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::sysops::backup_install_helpers::{CACHE_CLEANUP_ARGS, CACHE_CLEANUP_RULE};

    /// 追加命令必须与抽取前模板逐字一致，且规则与真正传给 sudo 的命令逐 token 一致。
    #[test]
    fn test_append_command_matches_rule_token_by_token() {
        let username = "tester";
        let line = format!("{username} ALL=(ALL) NOPASSWD: {CACHE_CLEANUP_RULE}");
        let cmd = build_sudoers_append_command(CACHE_CLEANUP_RULE, &line);

        let expected = format!(
            r#"sudo sh -c 'grep -qF "{rule}" {file} || printf "%s\n" "{line}" >> {file}; chmod 440 {file}'"#,
            rule = CACHE_CLEANUP_RULE,
            line = line,
            file = SUDOERS_FILE,
        );
        assert_eq!(cmd, expected, "追加命令模板必须与抽取前逐字一致");

        // -n 是 sudo 自身选项，不参与被授权命令匹配；其后必须逐 token 等于规则
        assert_eq!(CACHE_CLEANUP_ARGS[0], "-n");
        assert_eq!(
            CACHE_CLEANUP_ARGS[1..].join(" "),
            CACHE_CLEANUP_RULE,
            "sudoers 规则必须与真正传给 sudo 的命令逐 token 一致"
        );
    }

    /// 幂等 grep 的匹配串必须是完整规则，写入行必须包含用户名与规则原文。
    #[test]
    fn test_append_command_embeds_full_rule_and_username() {
        let line = format!("alice ALL=(ALL) NOPASSWD: {CACHE_CLEANUP_RULE}");
        let cmd = build_sudoers_append_command(CACHE_CLEANUP_RULE, &line);

        assert!(
            cmd.contains(&format!("grep -qF \"{CACHE_CLEANUP_RULE}\"")),
            "grep 幂等判断必须使用完整规则文本"
        );
        assert!(
            cmd.contains(&format!("printf \"%s\\n\" \"{line}\"")),
            "写入的整行必须与传入 line 逐字一致"
        );
        assert!(
            cmd.contains(">> /etc/sudoers.d/aur-helper-backup"),
            "必须追加写入，避免覆盖其它入口的规则"
        );
        assert!(cmd.ends_with("chmod 440 /etc/sudoers.d/aur-helper-backup'"));
        assert!(!cmd.contains("tee "), "禁止覆盖写");
    }
}
