/**
 * vite-env.d.ts - Vite 环境类型声明
 *
 * 功能：
 * - 提供 Vite 客户端类型引用
 * - 声明 .vue 文件的模块类型，使 TypeScript 能正确识别 Vue 单文件组件
 */
/// <reference types="vite/client" />

/** .vue 文件模块类型声明，确保 TypeScript 在导入 .vue 文件时不报错 */
declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
