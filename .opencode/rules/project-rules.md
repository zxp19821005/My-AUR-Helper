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

## 安全检查
- 不要在前端暴露数据库路径或配置密码
- AUR SSH 密钥使用系统 SSH agent，不硬编码在代码中
- 代理 URL 从远程 sourcescript 动态获取

## 验证
- 每次修改后运行 `cargo check` 确保 Rust 编译通过
- 运行 `pnpm vue-tsc --noEmit` 检查前端类型
- 使用 `cargo clippy` 检查 Rust 代码质量

## Git 提交规则
- **每次任务完成后必须及时提交并推送至 GitHub**
- 每次 commit 只包含同一任务的相关修改，避免一次 commit 混杂多个不相关任务
- commit message 需简要说明本次修改内容，中文或英文均可
