// build.rs — Tauri 构建脚本
// 在编译主 crate 之前由 Cargo 自动执行
// 负责处理 Tauri 特定的构建任务，如资源编译、图标生成、平台适配等
fn main() {
    // 调用 tauri_build::build() 执行 Tauri 标准构建流程：
    // - 解析 tauri.conf.json 配置
    // - 编译和嵌入资源文件
    // - 生成必要的绑定代码
    tauri_build::build()
}
