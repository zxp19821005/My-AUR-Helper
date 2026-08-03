mod download; // 下载代理文件子模块
/**
 * proxy/mod.rs - 代理管理模块
 *
 * 提供代理源的获取、下载、解析和延迟测试功能
 * 用于加速 AUR 包下载和 Git 操作
 */
mod fetch; // 从外部源获取代理列表子模块
mod parse; // 解析代理文件子模块
mod test; // 代理延迟测试子模块

pub use download::download_proxy_file; // 导出代理文件下载函数
pub use fetch::{fetch_proxy_list_from_userscript, FetchedProxy}; // 导出代理获取函数和数据结构
pub use parse::parse_proxy_file; // 导出代理文件解析函数
pub use test::{test_proxy_by_type, test_proxy_latency}; // 导出代理延迟测试函数
