/**
 * fe_log.rs - 前端诊断日志转发命令
 *
 * 功能：
 * - 接收前端（Vue / WebKitGTK WebView）发来的诊断日志
 * - 仅打印到 stdout（终端），【不写入文件日志】
 *
 * 用途：
 * dev 模式下 WebKitGTK 的 WebView 控制台输出不会转发到 Rust 进程终端，
 * 前端 console.log 在 `tauri dev` 终端中不可见。通过该命令把前端关键时序
 * （应用挂载、路由导航、数据请求起止）转发到终端，便于排查首屏白屏 / 卡顿。
 *
 * 注意：使用 println! 直接输出到 stdout，绕过后端 RotatingLogger，
 * 因此这些日志只会出现在终端，不会写入 applog 文件（符合"仅终端"要求）。
 */
use tauri::command;

/// 接收前端诊断日志并打印到终端（stdout，不写文件）。
///
/// @param level - 日志级别字符串（DEBUG / INFO / WARN / ERROR）
/// @param area - 前端模块 / 区域标识（如 main / App / Router / Dashboard）
/// @param message - 日志正文（前端已附加相对耗时等诊断信息）
/// @param ts - 前端本地时间戳（与后端日志同格式，便于跨进程对照）
#[command]
pub fn frontend_log(level: String, area: String, message: String, ts: String) {
    println!("[前端] {} {} [{}] {}", ts, level, area, message);
}
