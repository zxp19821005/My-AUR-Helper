# Rust 代码规则

## 代码规范

### 文件组织
- 单个文件不超过 300 行
- 超过 300 行必须拆分为多个独立文件
- 每个文件只负责一个功能模块
- 文件名需与功能模块统一

### 拆分规则
- `db/mod.rs` 按表拆分: `db/packages.rs`, `db/proxies.rs` 等

## 风格
- 使用 `cargo fmt` 格式化代码
- 遵循 Rust 2021 edition 规范
- 使用 `snake_case` 命名变量/函数/模块
- 使用 `PascalCase` 命名类型/trait/enum
- 使用 `SCREAMING_SNAKE_CASE` 命名常量

## 错误处理
- 使用 `anyhow::Result` 作为函数返回类型
- 使用 `thiserror` 定义领域错误类型
- 避免不必要的 `unwrap()` / `expect()` — 优先使用 `?` 传播错误
- Tauri command 函数捕获错误并转换为 `String` 返回

## 异步
- 使用 `tokio` 异步运行时
- I/O 密集型操作用 `async fn`
- CPU 密集型操作使用 `tokio::task::spawn_blocking`

## 数据库
- 所有数据库操作在 `db` 模块的独立文件中实现
- 使用 rusqlite prepared statement 防止 SQL 注入
- 数据库连接使用 `Mutex` 包装以共享状态

## 日志
- 使用 `tracing` crate 的宏：`info!`, `debug!`, `warn!`, `error!`
- 每条日志包含模块上下文信息

## Tauri
- 所有 IPC 命令在 `commands/` 模块中定义
- 命令函数签名：`async fn command_name(state: State<'_, AppState>, ...) -> Result<T, String>`
- 状态管理通过 `AppState` 结构体 + `app.manage()`
