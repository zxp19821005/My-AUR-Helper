/**
 * language.ts - 编程语言（Language）领域 API 封装
 *
 * 功能：
 * - 集中封装编程语言枚举的查询、新增/更新、删除等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { Language, EnumProgrammingLanguage } from "@/types";

/** 列出全部编程语言 */
export async function getLanguages(): Promise<Language[]> {
  return await invoke<Language[]>("get_languages");
}

/** 新增或更新编程语言（id 为 null 表示新增） */
export async function upsertLanguage(language: EnumProgrammingLanguage): Promise<void> {
  await invoke("upsert_language", { language });
}

/** 按名称删除编程语言 */
export async function deleteLanguage(name: string): Promise<void> {
  await invoke("delete_language", { name });
}
