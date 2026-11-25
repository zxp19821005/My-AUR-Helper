/**
 * fe_log.rs - 前端诊断日志转发命令
 *
 * 功能：
 * - 接收前端（Vue / WebKitGTK WebView）发来的诊断日志
 * - 同时打印到 stdout（终端）并按级别写入 applog 文件日志
 *
 * 用途：
 * dev 模式下 WebKitGTK 的 WebView 控制台输出不会转发到 Rust 进程终端，
 * 前端 console.log 在 `tauri dev` 终端中不可见。通过该命令把前端关键时序
 * （应用挂载、路由导航、数据请求起止）转发出来，便于排查首屏白屏 / 卡顿。
 *
 * 落盘：RotatingLogger 实现了 log::Log，log::*! 会写入 applog 文件；
 * target 固定为 "frontend" 便于 grep，前端本地时间戳保留在消息正文，
 * 与后端日志按同一时间轴对照，解决"只看 applog 看不到前端时序"的问题。
 */
/// 接收前端诊断日志：打印到终端 stdout，并按级别写入 applog 文件。
///
/// @param level - 日志级别字符串（DEBUG / INFO / WARN / ERROR）
/// @param area - 前端模块 / 区域标识（如 main / App / Router / Dashboard）
/// @param message - 日志正文（前端已附加相对耗时等诊断信息）
/// @param ts - 前端本地时间戳（与后端日志同格式，便于跨进程对照）
#[tauri::command]
#[specta::specta]
pub fn frontend_log(level: String, area: String, message: String, ts: String) {
    // 1) 终端输出：dev 模式下 WebKitGTK 的 WebView 控制台不会转发到 Rust 终端
    println!("[前端] {} {} [{}] {}", ts, level, area, message);

    // 2) 落盘：RotatingLogger 实现 log::Log，log::*! 会写入 applog 文件。
    //    target 固定为 "frontend"，便于 grep；前端时间戳保留在正文做跨进程对照。
    let line = format!("[前端] {} [{}] {}", ts, area, message);
    match level.as_str() {
        "ERROR" => log::error!(target: "frontend", "{}", line),
        "WARN" => log::warn!(target: "frontend", "{}", line),
        "DEBUG" => log::debug!(target: "frontend", "{}", line),
        _ => log::info!(target: "frontend", "{}", line),
    }
}
