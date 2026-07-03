# Vue 前端代码规则

## 代码规范

### 文件组织
- 单个文件不超过 300 行
- 超过 300 行必须拆分为多个独立文件
- 每个文件只负责一个功能模块
- 文件名需与功能模块统一

### 拆分规则
- 通用组件提取到 `src/components/`
- 页面组件保留在 `src/views/`
- 工具函数提取到 `src/utils/`
- API 调用封装到 `src/api/`

## 风格
- 使用 Vue 3 Composition API + `<script setup lang="ts">`
- TypeScript 严格模式，所有变量必须标注类型
- 组件名使用 PascalCase，文件名使用 PascalCase.vue
- 使用 Pinia 管理全局状态

## 组件规范
- 每个 .vue 文件只导出一个组件
- 模板中使用 `kebab-case` 事件名
- 使用 `scoped` 样式，避免全局污染
- CSS 变量在 `assets/styles.css` 中定义

## Tauri 通信
- 通过 `@tauri-apps/api/core` 的 `invoke()` 调用后端
- 类型定义在 `types/index.ts` 中，与 Rust models 保持同步
- 使用 Pinia store 封装 invoke 调用

## 路由
- 使用 `vue-router` 的 history 模式
- 路由定义在 `router/index.ts` 中
- 延迟加载页面组件

## 文件夹约定
- `src/views/` — 页面级组件（对应路由）
- `src/components/` — 可复用组件
- `src/stores/` — Pinia store 定义
- `src/types/` — TypeScript 类型定义
- `src/assets/` — 静态资源（CSS、图片）
