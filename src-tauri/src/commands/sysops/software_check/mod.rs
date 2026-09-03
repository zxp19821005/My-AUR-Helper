/**
 * software_check.rs - 版本检查命令入口
 */
pub mod selected;

pub use selected::{apply_upstream_check_result, check_selected_upstream};
