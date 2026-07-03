# Plan: 系统托盘设置功能

## 目标
在前端设置页面添加托盘相关设置，并在后端实现对应的逻辑。

## 新增设置项

| Key | 默认值 | 说明 |
|-----|--------|------|
| `show_tray_icon` | `true` | 是否显示系统托盘图标 |
| `close_action` | `minimize_to_tray` | 关闭窗口动作：`minimize_to_tray` 或 `exit` |

## 修改的文件

| 文件 | 修改内容 |
|------|----------|
| `src-tauri/src/lib.rs` | 读取设置决定是否显示托盘、关闭行为 |
| `src-tauri/src/db/mod.rs` | seed_defaults 中添加新设置默认值 |
| `src/views/Settings.vue` | 通用设置部分添加托盘相关 UI |

## 后端逻辑

### lib.rs setup 流程
1. 数据库初始化后，读取 `show_tray_icon` 和 `close_action` 设置
2. 如果 `show_tray_icon == "true"`，创建系统托盘
3. `on_window_event` 中根据 `close_action` 决定行为

### 设置读取
```rust
let show_tray = db.get_setting("show_tray_icon")
    .ok().flatten().map(|s| s.value == "true").unwrap_or(true);
let close_action = db.get_setting("close_action")
    .ok().flatten().map(|s| s.value).unwrap_or_else(|| "minimize_to_tray".to_string());
```

## 前端 UI

在 Settings.vue 的"通用设置"部分添加：
- 托盘图标开关（select: 显示/隐藏）
- 关闭动作选择（select: 最小化到托盘/直接退出）

## 验证
1. `cargo check` 编译通过
2. 运行应用测试：
   - 默认显示托盘图标
   - 关闭窗口最小化到托盘
   - 修改设置后重启生效
