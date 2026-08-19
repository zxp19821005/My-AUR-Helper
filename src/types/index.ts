/**
 * index.ts - 类型定义统一出口（barrel）
 *
 * 功能：
 * - 按领域拆分类型定义，本文件仅做再导出，保持原有 `@/types` / `../types` 导入路径兼容
 * - 领域模块：package（软件包）/ proxy（代理）/ enum（枚举）/ backup（备份）/ cache（缓存）/ settings（设置日志）/ dashboard（仪表盘）
 *
 * 注意：新增类型请写入对应领域文件，不要直接在本文件定义
 */

export * from "./package";
export * from "./proxy";
export * from "./enum";
export * from "./backup";
export * from "./cache";
export * from "./settings";
export * from "./dashboard";
