/**
 * ipc/mod.rs - IPC 命令注册与 bindings 导出模块入口
 *
 * 功能：声明 registry 子模块并统一再导出，供 lib.rs 以
 * `use ipc::registry::{build_specta_router, export_bindings}` 方式引用。
 *
 * 工作流程：
 * - registry 子模块承载唯一的 collect_commands! 注册点与导出函数
 * - 对外路径稳定，bindings 生成逻辑与 lib.rs 解耦
 *
 * 适用场景：Tauri 命令路由构建、TypeScript bindings 生成。
 */
pub(crate) mod registry;
