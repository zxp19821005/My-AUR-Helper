# 内存缓存管理系统设计方案（Memory Cache System）

> 日期：2026-08-20
> 范围：Rust 后端（src-tauri）+ Vue 前端（src）+ SQLite 持久化
> 状态：已评估，进入实施

---

## 1. 背景与问题

从运行日志观察，以下「低频变化的元数据」在每次页面访问时都触发全量 DB 查询：

| 场景 | 现状 | 数据量 |
|------|------|--------|
| 进入 License 管理页 | 每次 `get_licenses` 全表查询 | 762 条 |
| 进入语言管理页 | 每次 `get_languages` 全表查询 | 61 条 |
| 各页面读取设置 | 每次 `get_setting(s)` 查 `settings` 表 | ~20 条 |
| 软件详情 / 列表页 | 每包 3~5 次 DB 往返（虽有定向优化） | 1928 包 |

这些数据**写入频率远低于读取频率**（License/语言仅在枚举页或 SPDX 同步时变化；设置仅在用户保存时变化），
非常适合「内存缓存 + 磁盘持久化」：把读放大从 DB 移到内存，启动时一次加载，退出前落盘，重启后免重建。

## 2. 需求评估

用户需求（已确认）：

1. **内存缓存**：将常用数据（系统设置、License、编程语言等）加载到内存缓存
2. **缓存管理**：启动时创建缓存、数据更新时更新缓存、退出前将缓存写入硬盘
3. **缓存目录**：`~/.config/com.zxp19821005.aur-helper/cache`
4. **设置页**：新增「内存缓存管理」设置，含缓存大小、缓存有效期、缓存写入周期、缓存写入目录

评估结论：**可行，收益明确**。改动集中在命令层与新增 `cache/` 模块，对现有数据流侵入小；
复用项目已有的「设置表 + 草稿式设置页」模式，风险可控。

## 3. 总体架构

```
┌───────────────────────────────────────────────────────┐
│                 Vue 前端（设置页 + 枚举页）              │
│   api/cache.ts（getMemoryCacheStats / flush / clear）   │
└──────────────────────────┬────────────────────────────┘
                           │ Tauri IPC
┌──────────────────────────▼────────────────────────────┐
│                  Rust 后端（src-tauri）                 │
│  ┌─────────────────────────────────────────────────┐  │
│  │  commands/  （enums / settings / memory_cache）   │  │
│  │   └─ 读写操作后调用 cache.invalidate(...)         │  │
│  └────────────────────┬────────────────────────────┘  │
│                       ▼                                │
│  ┌─────────────────────────────────────────────────┐  │
│  │  cache/ 模块（新增）                              │  │
│  │  config.rs     ← 从 settings 表读缓存配置         │  │
│  │  manager.rs    ← CacheManager（内存主体，LRU）    │  │
│  │  persistence.rs← 磁盘读写（原子写 + JSON）        │  │
│  │  stats.rs      ← 缓存统计（供设置页展示）         │  │
│  └──────────────┬──────────────────┬───────────────┘  │
│                 │ 启动加载           │ 定时/退出落盘    │
│                 ▼                   ▼                 │
│          DB（SQLite）      ~/.config/.../cache/*.json  │
└───────────────────────────────────────────────────────┘
```

`AppState` 扩展：

```rust
pub struct AppState {
    pub db: Mutex<db::Database>,
    pub memory_cache: Mutex<CacheManager>,   // 新增
}
```

## 4. 缓存域设计（CacheDomain）

```rust
pub enum CacheDomain {
    Settings,   // 系统设置（get_setting / get_settings）
    Licenses,   // 全部 License 枚举
    Languages,  // 全部编程语言枚举
}
```

每个域一个内存条目；Licenses / Languages 为纯公开数据，支持落盘持久化，Settings 仅保留内存：

| 域 | 内存数据类型 | 磁盘文件 | 是否持久化 |
|----|--------------|----------|------------|
| Settings | `Vec<Setting>` | 无 | **否（仅内存缓存）** |
| Licenses | `Vec<EnumLicense>` | `licenses.json` | 是 |
| Languages | `Vec<EnumProgrammingLanguage>` | `languages.json` | 是 |

> **Settings 为何不落盘**：settings 表本身已持久化在 SQLite 中，磁盘再存一份收益为零；
> 且其中含 `github_token` / `gitee_token` / `gitlab_token` 等敏感凭据，落盘有明文泄露风险。
> 内存缓存已满足「减少 DB 读放大」的核心目标（get_setting/get_settings 高频调用）。
> 域采用枚举而非泛型表，便于命令层类型安全地读写；扩展新域只需加枚举变体 + 映射方法。

## 5. 配置项设计（settings 表，category = "memory_cache"）

| key | 默认值 | 说明 |
|-----|--------|------|
| `memory_cache_enabled` | `true` | 是否启用内存缓存（关闭后命令层直接走 DB） |
| `memory_cache_size` | `100` | 缓存条目上限（LRU 淘汰，域数量极少，主要防未来扩展） |
| `memory_cache_ttl` | `300` | 缓存有效期（秒），0 表示永不过期 |
| `memory_cache_write_interval` | `60` | 自动写盘周期（秒），0 表示关闭定时写（仅退出时写） |
| `memory_cache_dir` | `""` | 缓存写入目录；留空使用默认 `~/.config/com.zxp19821005.aur-helper/cache` |

`seed.rs` 补默认值；`memory_cache_dir` 展开 `~` 前缀（与 `log_dir` 现有逻辑一致）。

## 6. 生命周期

### 6.1 启动（setup）
1. 从 `database` 读取 5 项缓存配置 → 构造 `CacheConfig`
2. `CacheManager::load_from_disk()`：读取 3 个域文件，**未过期**的条目载入内存（过期条目直接丢弃）
3. 注册定时写盘任务（`tauri::async_runtime::spawn`，周期 = `write_interval`）
4. 将 `CacheManager` 与 `db` 一起 `app.manage(AppState)`

### 6.2 运行
- **读**：命令层调用 `cache_get_or_load(domain, || db_query())`——
  命中且未过期直接返回；miss 时执行闭包回源 DB 并填充内存（标记脏）
- **写**：`set_setting` / `add_license` / `upsert_language` 等写库成功后调用
  `cache.invalidate(domain)` 使对应域失效（下次读自动回源重建）

### 6.3 定时写盘
后台任务每 `write_interval` 秒 `flush()`：把**脏**条目原子写入磁盘（见 §7），成功后清脏标记。

### 6.4 退出
`lib.rs` 改为 `Builder::build()` + `.run(callback)`，在 `RunEvent::Exit` 中取 `AppState.memory_cache` 执行最终 `flush()`。
（托盘「退出」走 `app.exit(0)` 会触发该事件；窗口关闭动作仅隐藏窗口时应用仍在运行，由定时写盘兜底。）

## 7. 持久化格式与安全

文件格式（每个域一个文件）：

```json
{
  "meta": { "domain": "Licenses", "created_at": 1724145600, "expires_at": 1724145900, "size": 762 },
  "data": [ ... ]
}
```

- **原子写**：先写 `{domain}.json.tmp`，`fsync` 后 `rename` 覆盖，避免崩溃留下半截文件
- **过期处理**：读盘时 `expires_at < now` 视为 miss（不删除文件，由下次写覆盖）
- **安全（强制）**：`Settings` 域**不落盘**（见 §4）——settings 表含 `token`/`secret` 等敏感凭据，
  落盘有明文泄露风险（与 AGENTS.md「敏感信息禁止写入日志」同理）；
  Licenses / Languages 均为公开数据，落盘安全
- **目录安全**：目录路径来自设置，创建时 `create_dir_all`；文件名由 `CacheDomain` 枚举映射（白名单），
  不拼接用户输入，无路径遍历风险

## 8. Tauri 命令（commands/memory_cache.rs）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_memory_cache_stats` | - | `MemoryCacheStats` | 配置 + 各域状态（命中/大小/过期时间）+ 缓存目录 |
| `flush_memory_cache` | - | `usize` | 立即写盘（返回写入域数） |
| `clear_memory_cache` | - | `()` | 清空内存缓存与磁盘文件 |

`MemoryCacheStats` 模型加入 `models/memory_cache_stats.rs`。

## 9. 现有命令改造（写库后失效）

| 命令 | 改造 |
|------|------|
| `get_settings` / `get_setting` | 读走 Settings 缓存（enabled 时）；`set_setting` 写库后 `invalidate(Settings)` |
| `get_licenses` | 读走 Licenses 缓存 |
| `add_license` / `update_license` / `delete_license` / `sync_licenses_from_spdx` | 写库后 `invalidate(Licenses)` |
| `get_languages` | 读走 Languages 缓存 |
| `upsert_language` / `delete_language` | 写库后 `invalidate(Languages)` |

> 失效而非局部更新：枚举/设置数据量小（≤762 条），失效重建成本可忽略，且避免局部更新逻辑漂移。

## 10. 前端改动

1. **路由**：`src/router/index.ts` 新增 `/settings/memory-cache`（name `SettingsMemoryCache`）
2. **设置菜单**：`SettingsPopup.vue` 新增菜单项「内存缓存设置」（图标 `MemoryStick`，新加 `Icon.settingsMemoryCache`）
3. **分类分发**：`Settings.vue` 的 `categoryMap` 新增 `/settings/memory-cache` → `"memory_cache"`
4. **新组件**：`components/settings/SettingsMemoryCacheSection.vue`（草稿模型，仿 `SettingsLogSection`）：
   - 启用开关、缓存条目上限、有效期、写盘周期、写入目录（5 项设置，草稿 + 保存/重置）
   - 「缓存运行状态」卡片：读取 `get_memory_cache_stats` 展示各域命中/大小/过期时间、总占用、缓存目录
   - 操作按钮：立即写盘 / 清空缓存
5. **API**：`src/api/cache.ts` 新增 `getMemoryCacheStats` / `flushMemoryCache` / `clearMemoryCache`
6. **类型**：`src/types/cache.ts` 新增 `MemoryCacheStats` / `CacheDomainStats`

## 11. 实施步骤

1. `cache/` 模块（config / manager / persistence / stats / mod）
2. `models/memory_cache_stats.rs` + `models/mod.rs` 导出
3. `commands/memory_cache.rs` + 命令注册（lib.rs）
4. `AppState` 扩展 + `lib.rs` setup 初始化 + `RunEvent::Exit` 落盘改造
5. 现有命令（settings / enums）接入缓存与失效
6. `seed.rs` 补 5 项默认设置
7. 前端：类型 / api / 路由 / 菜单 / `Settings.vue` / `SettingsMemoryCacheSection.vue`
8. 验证：`cargo fmt` / `cargo clippy --lib` / `cargo test --lib` / `vue-tsc --noEmit` / `vite build`

## 12. 验证清单

- [ ] `cargo check --lib` 0 error
- [ ] `cargo clippy --lib` 0 warning
- [ ] `cargo test --lib` 全部通过
- [ ] `vue-tsc --noEmit` 0 error
- [ ] `vite build` 通过
- [ ] 功能冒烟：启动后进入枚举页不再重复全量查询（日志观察）；修改 License 后列表即时生效；退出后缓存文件生成；重启后命中磁盘缓存

## 13. 已知限制

- 缓存一致性为「写后失效」模型：外部进程直接改 DB 无法感知（本项目 DB 仅本应用写，可接受）
- `memory_cache_size` 目前实际约束 3 个域，主要面向未来扩展（如代理列表、统计）
- 若用户关闭缓存，命令层回退原 DB 路径，行为与现在完全一致（开关可随时回滚）
