// 阻止 Windows 发布版本中出现额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 应用入口函数
/// 调用 lib.rs 中的 run() 函数启动 Tauri 应用
fn main() {
    my_aur_helper_lib::run()
}
