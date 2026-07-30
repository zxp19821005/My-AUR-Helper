<!-- ================================================================ -->
<!-- My-AUR-Helper 项目开发计划文档                                      -->
<!-- 本文档定义了项目的开发路线图、代码规范、当前进度和架构模块关系      -->
<!-- 用于指导开发团队按计划迭代，确保代码质量和项目结构一致性            -->
<!-- ================================================================ -->

# My-AUR-Helper 项目开发计划

## 项目概述

Tauri 桌面应用，用于管理 AUR 软件包更新、本地备份和代理。

- **后端**: Rust (Tauri v2)
- **前端**: TypeScript + Vue 3
- **数据库**: SQLite (via rusqlite)
- **日志**: tracing + tauri-plugin-log

## 代码规范（强制）

### 文件组织原则
1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

### 拆分规则
- `commands/` 按职责分组: 顶层单文件命令 + `fileops/`（文件操作类）+ `sysops/`（系统操作类）
- `db/mod.rs` 按表拆分: `db/software_info.rs`, `db/aur_info.rs` 等
- Vue 组件拆分: 通用逻辑提取到 `composables/`，通用样式提取到 `assets/styles/` 全局 CSS

## 当前进度

- [x] 初始化 Rust 工具链
- [x] 搭建 Tauri + Vue 项目骨架
- [x] 实现基础功能模块
- [x] 拆分 db/mod.rs (630行 → 14个文件)
- [x] 拆分 commands/software.rs (537行 → 3个文件)
  - software.rs (163行) — CRUD 操作
  - software_sync.rs (246行) — AUR 同步
  - software_check.rs (144行) — 版本检查
- [x] 拆分 commands/files.rs (332行 → 2个文件)
  - files.rs (221行) — 文件操作
  - files_scan.rs (85行) — 包文件扫描
- [x] 提取 Vue 公共样式到全局 CSS
  - btn-icon 系列样式
  - modal 系列样式
  - form 系列样式
  - info-table 样式
- [x] 提取 PackageList.vue 操作逻辑到 composable
  - packageActions.ts (189行)
  - PackageList.vue 从 568行 减至 290行
- [x] 拆分超限 Vue 文件
  - [x] SoftwareFormModal.vue (353→179行) — 提取表单逻辑到 useSoftwareForm.ts (218行)
  - [x] PackageDetail.vue (320→237行) — 复用 useSoftwareForm.ts composable
- [x] 实现筛选器和上游 URL 验证功能
  - [x] 创建 upstream_validate.rs 命令 (130行)
  - [x] 创建 FilterBar.vue 组件 (220行)
  - [x] 更新 models (UpstreamUrlStatus 枚举, SoftwareListEntry 扩展字段)
  - [x] 更新 database schema (upstream_url_status 列)
  - [x] 更新 frontend types 和 composables (usePackageList 筛选逻辑)
  - [x] 更新 PageToolbar 支持 right slot
- [x] 重构备份管理模块
  - [x] 创建 BackupSoftwareEntry 模型和 DB 查询
  - [x] 重写 backup.rs 命令（列表/清空/扫描/去重/删除）
  - [x] 创建 useBackupList composable
  - [x] 重写 BackupManager.vue（匹配 PackageList 布局）
  - [x] 更新 frontend types
- [x] 前端组件目录模块化重组（2026-07-29）
  - [x] components/ 按功能拆分为 10 个子目录
  - [x] 样式集中到 assets/styles/
- [x] 统一双组件体系与代码质量修复（2026-07-30）
  - [x] 删除 Modal.vue / DataTable 系列，统一使用 Standardized 组件
  - [x] 拆分超限文件（PackageList/CacheManager/FilterBar/SettingsProxySection）
  - [x] 提取 toolbar-buttons.css / filter-styles.css
- [ ] 重构数据库 schema
  - [x] 更新 DATABASE.md 设计文档
  - [ ] 重写 models/mod.rs
  - [ ] 重写 db/ 层
  - [ ] 重写 commands/ 层
  - [ ] 更新前端 types/index.ts
  - [ ] 更新前端 views

## 项目架构

```
My-AUR-Helper/
├── docs/                    # 项目文档
│   ├── ARCHITECTURE.md      # 系统架构设计
│   ├── DATABASE.md          # 数据库设计
│   ├── API.md               # Tauri Command API
│   ├── COMPONENTS_GUIDE.md  # 前端组件使用指南
│   └── PLAN.md              # 项目开发计划
├── src/                     # Vue 前端
│   ├── views/               # 页面组件 (10个)
│   ├── components/          # 组件（按功能分 10 个子目录，50+ 文件）
│   ├── composables/         # 组合式函数 (12个)
│   ├── stores/              # Pinia 状态 (3个)
│   ├── utils/               # 工具函数 (3个)
│   ├── types/               # TypeScript 类型
│   └── assets/              # 静态资源（styles/ 集中管理组件样式）
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── lib.rs           # 库入口和命令注册
│   │   ├── main.rs          # 程序入口
│   │   ├── logger.rs        # 日志配置
│   │   ├── errors/          # 统一错误类型 (6个文件)
│   │   ├── models/          # 数据模型 (19个文件)
│   │   ├── db/              # SQLite 数据库 (23个文件)
│   │   ├── commands/        # Tauri IPC 命令（顶层 + fileops/ + sysops/）
│   │   ├── checkers/        # 版本检查器（含 github/ 子模块）
│   │   ├── versions/        # 版本处理（解析/标准化/比较）
│   │   ├── aur/             # AUR 交互 (3个文件)
│   │   └── proxy/           # 代理管理 (5个文件)
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## 数据库设计 (SQLite)

参见 [DATABASE.md](DATABASE.md)

核心表：
- `software_info` — 软件核心信息
- `aur_info` — AUR 包详细信息
- `upstream_info` — 上游版本信息
- `backup_software` — 备份文件
- `cache_software` — 缓存文件
- `proxies_info` — 代理信息
- `proxies_test` — 代理测试结果
- `enum_licenses` — 许可证枚举
- `enum_programming_languages` — 编程语言枚举
- `logs` — 应用日志
- `settings` — 应用设置

## 模块依赖关系

```
Tauri Commands (commands/ = 顶层 + fileops/ + sysops/)
    ↓
  AUR 模块 (aur/) ← 代理模块 (proxy/)
    ↓
 检查器模块 (checkers/) ← 代理模块 (proxy/)
    ↓
 版本处理模块 (versions/)
    ↓
   数据库模块 (db/) → SQLite
    ↓
   日志模块 (logger) → 文件/控制台

错误处理：所有模块统一使用 errors/ 的 AppError / AppResult
```

## 开发命令

```bash
pnpm install       # 安装前端依赖
pnpm tauri dev     # 开发模式
pnpm tauri build   # 构建生产版本
cargo check        # Rust 类型检查
cargo clippy       # Rust lint 检查
cargo fmt          # Rust 代码格式化
pnpm vue-tsc --noEmit  # TypeScript 类型检查
```
