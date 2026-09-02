# GitHub tags 缓存系统实现

## 问题
electron10-bin / electron2-bin 等老版本包需要翻数百页 tags，37 个 electronXX-bin 包每次检查重复请求同一仓库数千次。

## 解决方案
SQLite 表 `github_tag_cache`，按 `(owner, repo)` 存 tags/releases 快照，TTL 默认 24h。
检查前查缓存，命中则跳过整个仓库的网络请求。

## Phase 4：增量校验（2026-09-02）
缓存过期时不全量重拉，改为轻量验证：
1. 调 `GET /repos/{owner}/{repo}/releases/latest`（一次 HTTP 请求）
2. 与 `cached_version` 字段比对
   - 一致 → 顺延 TTL（Extend），跳过网络请求
   - 不一致 → 用缓存的 tags JSON 重算版本（不调网络），更新 cached_version（Recalculate）
   - 仓库不可访问 → 清除缓存（Deleted）
   - 无缓存 → 全量重拉（Miss）

`check_and_extend_cache` 为同步函数（`reqwest::blocking::get`），因 `Database` 不可 `Send`，调用侧在 `state.db.lock()` 内串行执行。

## 新增 IPC 命令
- `clear_github_tag_cache` — 清除全部 GitHub tags 缓存
- `clear_expired_github_tag_cache` — 清除过期缓存
- `get_github_tag_cache_stats` — 返回缓存条数和 TTL

## 修改文件（13 个）
- `src/db/github_tag_cache.rs`（新建）— CRUD + CacheCheckResult 枚举 + recompute_version_from_cache + check_and_extend_cache
- `src/db/migration_github_tag_cache.rs`（新建）— ALTER TABLE 迁移
- `src/db/schema.rs` — CREATE TABLE 加 cached_version 列
- `src/db/mod.rs` — pub(crate) mod
- `src/db/connection.rs` — initialize 调用迁移
- `src/commands/sysops/software_sync/batch.rs` — query_valid_github_caches + on_cache_ready + write_github_tag_cache 传 cached_version
- `src/commands/sysops/software_sync/upstream.rs` — skip_keys 增量校验逻辑
- `src/commands/sysops/software_sync/software_check.rs` — 空回调
- `src/commands/sysops/cache_cleanup.rs` — 3 个新命令
- `src/commands/sysops/mod.rs` — 导出
- `src/lib.rs` — 注册命令
- `Cargo.toml` — reqwest 加 blocking feature

## 验证
- `cargo check` → 0 warnings
- `cargo test --lib` → 83 passed
- `npx vue-tsc` → 0 错误

## 注意事项
- Database 不可 Send，跨 await 传引用会编译报错；用 on_cache_ready 回调在 batch.rs 内部同步调用
- 缓存命中时跳过整个仓库的所有包，不只跳过 tags 翻页
- check_and_extend_cache 是同步函数，HTTP 用 reqwest::blocking；调用侧用 state.db.lock() 包裹后串行执行
