<!-- ================================================================ -->
<!-- 架构演进与长期规划（ADR 集合）                                    -->
<!-- 记录超出当前迭代的、影响代码组织的重大架构决策。                    -->
<!-- 每个 ADR 独立成节，按时间倒序排列。                                 -->
<!-- ================================================================ -->

# 架构演进与长期规划

本文档记录**影响系统代码组织与运维形态的重大架构决策**（ADR — Architecture Decision Record），目的是让未来的我们（或接手者）能快速理解"为什么是现在这个样子，而不是别样"。

> 与 `ARCHITECTURE.md` 的区别：`ARCHITECTURE.md` 描述**当前**架构的事实；本文档描述**为何走向当前架构**以及**未来如何演化**。

---

# ADR-001: 按任务节奏渐进拆分本地与云端（GitHub Actions）

## Status
**Proposed**（2026-08-27）

## Context

当前系统是**单进程桌面应用**（Tauri + SQLite + reqwest），所有功能跑在本机：

- 后端：Rust（`src-tauri/src/`），按职责分为 `commands/`、`checkers/`、`db/`、`versions/` 等模块
- 前端：Vue 3 + TypeScript
- 持久化：本地 SQLite（`~/.config/com.zxp19821005.aur-helper/my_aur_helper.db`）

经 domain discovery（详见下方 §Domain Discovery），识别出三大功能域：

| 功能域 | 主要职责 | 设备绑定 | 资源消耗 |
|--------|---------|---------|---------|
| 系统集成（`sysops/` + `fileops/`）| pacman -Qip、sudoers 校验、本地 .pkg.tar.zst 扫描、缓存目录读写 | ✅ 强（必须本机用户态）| 中等 |
| 版本/上游检查（`software_check.rs` + `software_sync/upstream.rs` + `upstream_validate.rs`）| 调 GitHub/Gitee/GitLab/HTTP API 检查 1000+ 包的上游版本 | ❌ 纯网络 | 🔴 高（长时间 + 大量并发 HTTP） |
| 元数据/统计（`enums.rs` + `dashboard.rs` + `memory_cache.rs`）| License/Language 缓存、仪表盘聚合 | 部分（统计基于本地 DB） | 低 |

**用户反馈的真实痛点**（2026-08-27 架构咨询）：
> "多个检查任务叠加（CPU/网络扎堆）"

具体表现：
- 上游版本检查（1000+ 包 × GitHub GraphQL/REST）+ AUR 同步 + 上游 URL 验证 三个任务**同时**或短时间内运行，互相抢 CPU/网络/reqwest 连接池
- 本地必须保留的 pacman/sudoers/文件操作与"长时间重任务"在**同一个进程**内调度
- 用户的本机（Intel i915 + X11 + 内存压力）一旦 webkit2gtk 全页面卡顿，**全部功能不可用**（Tauri 进程被阻塞）

**自用系统**（非团队/非跨设备），无需实时双向同步，但希望"重/慢"任务从本机卸载，让本机保持响应。

## Decision

**采用方案 A：按任务节奏的渐进拆分**，将"长时间、高延迟、易爆栈"的任务云化，**保留"短平快、用户态、设备绑定"的任务本地**。

### 拆分原则（不可妥协）

| 任务 | 节奏 | 去向 | 触发方式 |
|------|------|------|---------|
| 上游版本检查 | 慢（5-30 min）/网络重 | 🟢 **云端** | GitHub Action `workflow_dispatch` + `schedule` |
| AUR 同步 | 中（5-15 min） | 🟢 **云端** | GitHub Action 监听 aur.archlinux.org RSS → `repository_dispatch` |
| 上游 URL 验证 | 慢 | 🟢 **云端** | 同上 |
| 缓存/枚举/统计 | 快（< 1 s）| 🔵 **本地** | 实时 |
| 备份/缓存管理（pacman/sudoers/本地文件）| 设备绑定 | 🔴 **本地必须** | 实时 |
| CRUD/设置/代理探测 | 设备绑定 | 🔴 **本地必须** | 实时 |

### 架构形态

```
┌─────────────────────┐         ┌──────────────────────────────┐
│  GitHub Actions     │  push   │  私有 mirror repo             │
│  (Linux runner)     │ ──────► │  /upstream-results/{date}.json│
│                     │         │  (atomic commit)              │
│  复用 src-tauri/    │         └──────────┬───────────────────┘
│  /checkers/         │                    │ git pull (本地触发)
│  /db/  (只读子集)   │                    ▼
│  /versions/         │         ┌──────────────────────────────┐
│                     │         │  桌面应用 (Tauri)              │
└─────────────────────┘         │  - commands/cloud_sync.rs     │
                                │  - git pull + JSON parse      │
                                │  - merge 到本地 SQLite         │
                                └──────────────────────────────┘
```

### 关键技术决策

1. **代码组织**：新增 `src-tauri/crates/{shared,cloud-checker}/`，通过 **Cargo workspace path 依赖** 共享 `checkers/` `versions/` `db/` 子集，**禁止复制代码**
2. **触发器**：`workflow_dispatch`（GitHub 网页手动按钮）+ `schedule`（每日凌晨 cron）+ `repository_dispatch`（aur RSS 变化触发）
3. **结果存储**：原子 JSON（先写 `.tmp` 再 rename），commit 到私有 mirror repo 的 `upstream-results/{YYYY-MM-DD}.json`
4. **本地拉取**：新增 `cloud_sync` 命令 → `git pull --ff-only` → parse JSON → 与本地 SQLite 做"按 `updated_at` 较新的覆盖合并"
5. **网络代理**：云端用 GitHub Actions runner 自带 IP（无需代理）；本地请求仍走原本的代理设置
6. **数据安全**：私有 mirror repo + GitHub Actions secrets 管理 GitHub token；本地 SQLite 不直接暴露

### 实施 Phase（渐进）

| Phase | 内容 | 周期 | 价值 |
|-------|------|------|------|
| **P1** | `crates/cloud-checker/` 独立 Rust binary，复用 shared crate，GitHub Action 写 JSON 到 mirror | 2-3 d | 解决 80% 痛点（上游检查）|
| **P2** | 桌面端 `cloud_sync` 模块：`git pull` + JSON 解析 + 与本地 SQLite 合并 | 1-2 d | 结果回流 |
| **P3** | `workflow_dispatch` 手动触发 UI + dashboard 展示"上次云端检查时间" | 0.5 d | 用户感知 |
| **P4** | AUR 同步走云端 + aur.archlinux.org RSS 触发 | 1-2 d | 进一步减负 |

## Consequences

### 变得容易

- ✅ **痛点直接消除**：长时间重任务从本机卸载，本机保持响应
- ✅ **代码复用**：复用现有 `checkers/` `versions/` 代码（path deps，无代码重复）
- ✅ **部署简单**：GitHub Actions + 私有 repo，**无新基础设施**（无 DB / 无 K8s / 无 CI 服务）
- ✅ **渐进推进**：可分 P1-P4 推进，每步独立可回滚
- ✅ **可观测性**：GitHub Actions 天然有日志、状态、审计
- ✅ **可逆性**：Phase 1 失败回滚成本低（删 `crates/cloud-checker/` 与 `.github/workflows/` 即可）

### 变得困难

- ❌ **数据合并是最终一致的**（不是实时）—— 用户需理解"先云端跑，再本地拉"
- ❌ **私有 mirror repo 仓库体积会随结果增长**（需定期归档旧 JSON）
- ❌ **桌面应用新增 `cloud_sync` 模块**与 git pull 依赖（约 200-300 行新代码）
- ❌ **GitHub Actions 运行时间消耗免费额度**（公开 repo 无限制，私有 repo 2000 min/月对个人够用）
- ❌ **共享 crate 拆分后跨 binary 重编译代价**（path deps 改动会触发下游重编，但 Rust 增量编译可缓解）
- ❌ **不适用跨设备实时场景**（如未来需要团队协作或多设备实时同步，方案 B 是 escape hatch）

## Alternatives Considered

### 方案 B：完全自建中央服务

把方案 A 的"私有 mirror repo + git pull"换成"轻量 HTTP API"（Cloudflare Worker + D1 / Fly.io + Postgres）。

- **优势**：实时双向、跨设备、可推送通知
- **代价**：新基础设施（DB + API + 鉴权 + 部署）+ 月度云资源开销 + 维护成本
- **否决理由**：**对自用系统严重 over-engineering**（违反 *"No architecture astronautics"*），相当于用企业架构解决个人问题。**保留为 escape hatch**：若未来需要跨设备实时性，可从方案 A 增量迁移到方案 B（mirror repo 仍可作为离线 fallback）。

### 方案 C：全本地优化（多线程/缓存/cron）

不加云端，纯粹在本机用多线程/优先级队列/cron 错峰解决扎堆。

- **优势**：架构最简，无新组件
- **代价**：治标不治本，本机资源上限是物理的；用户本机内存/webkit2gtk 问题未解决
- **否决理由**：用户痛点"多个任务叠加"本质是**资源上限问题**，全本地无法突破物理限制

### 方案 D：完全云化

所有逻辑上云，本地仅做"展示 + 触发"。

- **否决理由**：违反"pacman/sudoers/本地文件必须本机"硬约束，**系统集成**功能不可云化

## Reversibility

方案 A 的回滚成本低：
- 删 `.github/workflows/` + `src-tauri/crates/cloud-checker/` + 桌面端 `cloud_sync` 命令即可恢复纯本地
- 私有 mirror repo 可归档保留（不影响主系统）

如未来需要跨设备实时性，可从方案 A 增量演化到方案 B（保留 mirror repo 作为离线 fallback）。

---

# Domain Discovery（决策背景）

> 此节为辅助上下文，记录 ADR-001 决策时所依据的代码现状盘点。

## 当前 commands/ 目录分类

```
src-tauri/src/commands/
├── dashboard.rs          # 仪表盘聚合统计（快，可云）
├── enums.rs              # License/Language 枚举 CRUD（快，本地）
├── fe_log.rs             # 前端诊断日志（本地）
├── logs.rs               # 日志查询（本地）
├── memory_cache.rs       # 内存缓存统计（快，本地）
├── settings.rs           # 设置读写（本地）
├── software.rs           # 软件包 CRUD（本地）
├── proxy/                # 代理配置 + 连通性测试
├── fileops/              # 本地文件操作（cache_scan, backup_dedup, cache_backup）
└── sysops/               # 系统集成（pacman, sudoers, cache_install, software_check, software_sync）
    ├── software_check.rs       # 🟢 上游版本检查（重，云化目标）
    ├── software_sync/          # 🟢 AUR 同步 + check_all_upstream（重，云化目标）
    │   ├── aur.rs
    │   ├── upstream.rs
    │   ├── batch.rs
    │   ├── pkgbuild.rs
    │   └── ...
    ├── upstream_validate.rs    # 🟢 上游 URL 验证（云化目标）
    ├── cache_install.rs        # 🔴 设备绑定（pacman + sudoers）
    ├── cache_cleanup.rs        # 🔴 设备绑定（系统 pacman 操作）
    ├── backup_install.rs       # 🔴 设备绑定
    ├── pacman_lock.rs          # 🔴 设备绑定
    ├── proxy_utils.rs          # 🔴 探测本地代理端口
    └── ...
```

## 关键结论

- **"版本/上游检查"是云端化的最大收益点**——纯网络 + 内存比较，1000+ 包的 GitHub GraphQL/REST 调用是最大资源消费者
- **"系统集成"必须本地**——pacman/sudoers/本地文件都是用户态系统调用，离开本机无法工作
- **"CRUD/统计/枚举"保留本地**——延迟敏感、依赖本地 SQLite 状态
- **共享代码已就位**：`checkers/` `versions/` `db/` 模块化清晰，Cargo workspace path 依赖拆分成本低

---

# 未来 ADR 候选（待评估）

| 主题 | 触发条件 | 优先级 |
|------|---------|--------|
| 跨设备实时同步（方案 B 演化）| 多设备使用 / 团队协作需求 | 低 |
| 本地 DB 加密（SQLCipher）| 笔记本丢失场景增多 | 中 |
| 桌面应用国际化（i18n）| 分享给社区使用 | 低 |
| Tauri 升级（v2 → v3）| Tauri v3 稳定后 | 跟踪 |
| Web 端版本（脱离 Tauri）| 移动端需求 | 低 |
