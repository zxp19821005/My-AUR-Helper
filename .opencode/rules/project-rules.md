# 项目全局规则

## 代码规范（强制）

### 文件组织原则
1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

### 拆分规则
- `db/mod.rs` 按表拆分: `db/packages.rs`, `db/proxies.rs` 等
- Vue 组件拆分: 通用组件提取到 `src/components/`

## 文件组织
- `docs/` — 项目文档（架构、数据库、API）
- `src-tauri/src/` — Rust 后端代码
- `src/` — Vue 前端代码
- `.opencode/` — opencode AI 配置

## 修改顺序
1. 先修改 Rust 后端 models（如果需要）
2. 再修改 db 层
3. 然后修改 commands 层
4. 最后修改 Vue 前端类型和组件

## 数据库迁移
- 数据库 schema 通过 `db::Database::initialize()` 中的 `CREATE TABLE IF NOT EXISTS` 管理
- 新增表直接追加，不要修改已有表结构避免破坏已有数据
- 如需修改已有表，使用 `ALTER TABLE` 语句

## 数据库表结构（关键表）
### aur_info（AUR 包详情）
| 字段 | 类型 | 说明 |
|------|------|------|
| software_id | INTEGER PK FK | 关联 software_info.software_id |
| pkgdesc | TEXT | 软件包描述 |
| aur_version | TEXT | AUR 中的版本号 |
| license_id | INTEGER FK | 关联 enum_licenses.id |
| last_updated | INTEGER | AUR 最后更新时间 (Unix 时间戳) |
| depends | TEXT | 运行依赖 (JSON 数组) |
| makedepends | TEXT | 编译依赖 (JSON 数组) |
| optdepends | TEXT | 可选依赖 (JSON 数组) |
| out_of_date | INTEGER | AUR 标记是否过期 |

## 安全检查
- 不要在前端暴露数据库路径或配置密码
- AUR SSH 密钥使用系统 SSH agent，不硬编码在代码中
- 代理 URL 从远程 sourcescript 动态获取

## 验证
- 每次修改后运行 `cargo check` 确保 Rust 编译通过
- 运行 `pnpm vue-tsc --noEmit` 检查前端类型
- 使用 `cargo clippy` 检查 Rust 代码质量
