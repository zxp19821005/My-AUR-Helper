/**
 * software_info_sync.rs - 按维护者从 AUR 同步时的 software_info 写入逻辑
 *
 * 功能：
 * - 提供「存在则更新、不存在则新建」的 upsert，专门服务按维护者批量导入场景
 * - 关键约定：无论新建还是更新，都不触碰 upstream_url
 *   - 新建包：upstream_url 留空（AUR 只提供项目主页，不是上游版本检查地址）
 *   - 已存在包：保留用户手动配置的上游地址，AUR 同步不得覆盖
 * - 已存在包只更新 AUR 可提供/可推断的字段（包类型、检查器类型、测试版本/二进制标记），
 *   不直接写入 is_outdated（改由 recompute_outdated_from_aur 依据版本比较重算），
 *   也不动用户偏好字段，避免全量覆盖造成数据丢失
 *
 * 使用场景：
 * - 从 AUR 按 maintainer/comaintainers 拉取全部包并导入本地数据库
 *
 * 为什么独立成文件：
 * - software_info.rs 已达 296 行，逼近项目规定的 300 行上限，
 *   故将同步专用写入逻辑拆到本文件，符合单一职责与行数约束。
 */
use super::Database;
use crate::errors::AppResult;
use crate::models::SoftwareInfo;
use rusqlite::params;

impl Database {
    /// 按 AUR 同步语义写入软件包记录（不改上游地址）
    ///
    /// - 包已存在：更新除 `upstream_url` / `software_id` / `pkgname` 之外的全部字段，
    ///   返回原 `software_id`；上游地址、软件 ID、包名保持原值。
    /// - 包不存在：新建记录，`upstream_url` 强制留空，返回新 `software_id`。
    ///
    /// # 参数
    /// - `sw`: 待写入的软件包信息；其 `upstream_url` 在新建分支会被忽略并置空
    ///
    /// # 返回
    /// - `Ok((software_id, created))`: 软件 ID 与是否新建（true=新建，false=更新）
    pub fn upsert_software_from_aur(&self, sw: &SoftwareInfo) -> AppResult<(i64, bool)> {
        if let Some(existing) = self.get_software_by_name(&sw.pkgname)? {
            let sid = existing.software_id.ok_or_else(|| {
                crate::errors::AppError::ConfigError(format!(
                    "软件包 '{}' 存在但缺少 software_id",
                    sw.pkgname
                ))
            })?;
            // 已存在：只更新「AUR 可提供或从包名可推断」的字段。
            // 保留 upstream_url（用户配置的上游地址）、is_outdated（上游检查派生状态，
            // 重置会导致软件更新页丢失待更新标记）、language_id / version_extract_regex /
            // auto_check_enabled / skip_check_upstream（AUR 无数据源的用户偏好），
            // 避免全量覆盖造成的数据丢失。
            self.conn.execute(
                "UPDATE software_info SET
                    package_type_id=?1, checker_type_id=?2,
                    check_test_versions=?3, check_binary_files=?4
                 WHERE software_id=?5",
                params![
                    sw.package_type_id.as_id(),
                    sw.checker_type_id.as_id(),
                    sw.check_test_versions as i32,
                    sw.check_binary_files as i32,
                    sid,
                ],
            )?;
            Ok((sid, false))
        } else {
            // 新建：upstream_url 强制留空，AUR 的 URL 字段不是上游版本检查地址
            let language_ids_json = serde_json::to_string(&sw.language_ids).unwrap_or_default();
            self.conn.execute(
                "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id,
                    is_outdated, check_test_versions, check_binary_files,
                    auto_check_enabled, skip_check_upstream, language_id, version_extract_regex)
                 VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    sw.pkgname,
                    sw.package_type_id.as_id(),
                    sw.checker_type_id.as_id(),
                    sw.is_outdated as i32,
                    sw.check_test_versions as i32,
                    sw.check_binary_files as i32,
                    sw.auto_check_enabled as i32,
                    sw.skip_check_upstream as i32,
                    language_ids_json,
                    sw.version_extract_regex,
                ],
            )?;
            Ok((self.conn.last_insert_rowid(), true))
        }
    }

    /// 依据同步到的 AUR 版本重算 `software_info.is_outdated`
    ///
    /// 与主线同步 `sync_from_aur` / `update_aur_info` 同源，统一调用
    /// `versions::is_outdated_with_aur_signal`：主判定为版本比较（AUR 版本落后
    /// 于上游即为过期），版本相同时以 AUR 的 OutOfDate 标记作补充信号。
    ///
    /// 仅当 AUR 版本与上游版本都就绪时才重算，避免无依据翻转标记；
    /// 新建包没有上游版本，调用后不会改动 `is_outdated`（保持默认 false）。
    ///
    /// # 参数
    /// - `software_id`: 软件包 ID
    /// - `aur_version`: 本次同步到的 AUR 版本（可为空）
    /// - `aur_flagged`: AUR RPC 的 OutOfDate 标记
    /// - `pkgname`: 包名，仅用于日志定位
    pub fn recompute_outdated_from_aur(
        &self,
        software_id: i64,
        aur_version: Option<&str>,
        aur_flagged: bool,
        pkgname: &str,
    ) -> AppResult<()> {
        let up_ver = self
            .get_upstream_info(software_id)?
            .and_then(|u| u.upstream_version)
            .filter(|s| !s.is_empty());
        if let (Some(aur_ver), Some(up_ver)) = (aur_version, up_ver.as_deref()) {
            let is_outdated =
                crate::versions::is_outdated_with_aur_signal(aur_ver, up_ver, aur_flagged, pkgname);
            self.update_software_outdated(software_id, is_outdated)?;
        }
        Ok(())
    }
}
