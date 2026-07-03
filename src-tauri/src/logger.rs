// 日志宏模块 — 封装 log crate 的宏，与 tauri-plugin-log 集成
// 提供便捷的日志输出宏，默认日志框架使用 log crate

/// 输出 DEBUG 级别日志
/// 使用方式：log_debug!("message {}", arg);
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

/// 输出 INFO 级别日志
/// 使用方式：log_info!("message {}", arg);
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

/// 输出 WARN 级别日志
/// 使用方式：log_warn!("message {}", arg);
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*)
    };
}

/// 输出 ERROR 级别日志
/// 使用方式：log_error!("message {}", arg);
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}
