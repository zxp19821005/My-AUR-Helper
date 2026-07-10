/**
 * proxy/mod.rs - 代理管理模块
 *
 * 提供代理源的获取和延迟测试功能
 * 用于加速 AUR 包下载和 Git 操作
 */
mod fetch; // 从外部源获取代理列表子模块
mod test; // 代理延迟测试子模块

pub use fetch::{fetch_proxy_list_from_userscript, FetchedProxy}; // 导出代理获取函数和数据结构
pub use test::test_proxy_latency; // 导出代理延迟测试函数
