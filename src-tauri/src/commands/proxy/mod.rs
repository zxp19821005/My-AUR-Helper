/**
 * proxy/mod.rs - 代理管理命令模块
 *
 * 负责模块声明与导出。具体命令按职责拆分到子模块：
 * - basic.rs: 代理的增删改查与文件解析命令
 * - test.rs:  代理连通性测试相关命令与辅助函数
 *
 * 通过 `pub use` 将子模块公共项重新导出到 `commands::proxy` 命名空间，
 * 以保持 `lib.rs` 中 `commands::proxy::xxx` 的注册路径不变。
 */
pub mod basic;
pub mod test;

// 重新导出子模块的公共命令与类型
pub use basic::*;
pub use test::*;
