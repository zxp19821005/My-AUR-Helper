/**
 * packages.ts - 软件包状态管理（Pinia Store）
 *
 * 功能：
 * - 管理软件包列表状态
 * - 提供获取软件包列表方法
 * - 提供检查上游版本方法
 *
 * 数据来源：
 * - list_software: 获取所有软件包
 * - check_upstream_version: 检查上游版本
 */
import { defineStore } from "pinia";              // Pinia 状态管理库，用于创建 Store
import { ref, shallowRef } from "vue";            // Vue 响应式 API，shallowRef 承载千级大数组
import type { SoftwareInfo } from "../types";     // 软件包类型定义
import * as softwareApi from "@/api/software";    // 软件包领域 API

/** 创建 packages Store，用于管理软件包相关的全局状态 */
export const usePackageStore = defineStore("packages", () => {
  /** 软件包列表 - 存储所有软件包的浅响应式数组。
   *  注意：1932 个对象深度响应式代理会造成赋值时秒级阻塞，
   *  本 store 只做整体替换与只读统计（length/filter），shallowRef 足够。 */
  const packages = shallowRef<SoftwareInfo[]>([]);

  /** 加载中状态 - 标识是否正在加载数据 */
  const loading = ref(false);

  /** 错误信息 - 存储操作过程中的错误信息，null 表示无错误 */
  const error = ref<string | null>(null);

  /**
   * 获取软件包列表
   * 调用后端 list_software 命令，从数据库加载所有软件包
   * 加载前后自动管理 loading 状态
   */
  async function fetchPackages() {
    loading.value = true;
    error.value = null;
    try {
      packages.value = await softwareApi.listSoftware();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /**
   * 检查指定软件包的上游版本
   * 调用后端 check_upstream_version 命令获取上游最新版本
   * @param pkgname - 软件包名称
   * @returns 上游版本号字符串
   */
  async function checkVersion(pkgname: string): Promise<string> {
    return await softwareApi.checkUpstreamVersion(pkgname);
  }

  return { packages, loading, error, fetchPackages, checkVersion };
});
