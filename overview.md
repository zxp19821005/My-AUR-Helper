# AUR 搜索与导入功能实现

## 问题
用户反馈"从 AUR 同步"按钮无法同步新提交的包（如 `electron45-bin`）。原因：`sync_from_aur` 只遍历本地已存在的包名更新 `aur_info`，不导入新包。

## 解决方案
新增 AUR 搜索 + 导入功能，让用户可以手动搜索 AUR 并导入新包。

## 修改文件

### 后端（Rust）
| 文件 | 改动 |
|------|------|
| `src-tauri/src/aur/rpc.rs` | 新增 `search_packages(keyword)` 函数 |
| `src-tauri/src/aur/mod.rs` | 导出 `search_packages` |
| `src-tauri/src/commands/sysops/software_sync/import_aur.rs` | 新建，157 行 |
| `src-tauri/src/commands/sysops/software_sync/mod.rs` | 添加模块声明和导出 |
| `src-tauri/src/lib.rs` | 注册两个新命令 |

### 前端（Vue/TypeScript）
| 文件 | 改动 |
|------|------|
| `src/api/software.ts` | 新增 `searchAurPackages` 和 `importAurPackage` |
| `src/types/package.ts` | 新增 `AurSearchResult` 接口 |
| `src/views/PackageList.vue` | 添加工具栏按钮 + 搜索导入对话框 |

## 新功能
1. **工具栏新增"从AUR导入新包"按钮**（下载图标，青色）
2. **搜索对话框**：输入包名关键词 → 搜索 → 展示匹配结果（包名/版本/描述）→ 点击"导入"
3. **自动推断**：`-bin`/`-appimage` → 二进制包(GitHubAPI)；`-git` → Git仓库；其他 → 编译安装(手动检查器)
4. **Toast 反馈**：导入成功/失败均有提示

## 验证
- `cargo check` ✅ 0 错误 0 警告
- `vue-tsc --noEmit` ✅ 通过
