/**
 * settings.ts - 应用设置状态管理
 *
 * 功能：
 * - 管理应用设置的响应式状态
 * - 提供获取设置值的方法
 * - 支持设置值的实时更新
 */
import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** Setting 类型定义 */
interface Setting {
  id?: number;
  key: string;
  value: string;
  description?: string;
  category: string;
  created_at?: string;
}

export const useSettingsStore = defineStore("settings", () => {
  /** 设置缓存 - 键值对形式存储所有设置 */
  const settingsCache = ref<Record<string, string>>({});

  /** 获取设置值（始终从数据库读取，避免跨窗口缓存不一致） */
  async function getSetting(key: string, defaultValue: string = ""): Promise<string> {
    try {
      const setting = await invoke<Setting | null>("get_setting", { key });
      if (setting && setting.value) {
        settingsCache.value[key] = setting.value;
        return setting.value;
      }
      return defaultValue;
    } catch {
      return defaultValue;
    }
  }

  /** 获取数字类型的设置值 */
  async function getSettingNumber(key: string, defaultValue: number = 50): Promise<number> {
    const value = await getSetting(key, String(defaultValue));
    return parseInt(value, 10) || defaultValue;
  }

  /** 设置值并更新缓存 */
  async function setSetting(key: string, value: string): Promise<void> {
    await invoke("set_setting", { key, value });
    settingsCache.value[key] = value;
  }

  /** 刷新指定设置的缓存 */
  async function refreshSetting(key: string): Promise<void> {
    try {
      const setting = await invoke<Setting | null>("get_setting", { key });
      if (setting && setting.value) {
        settingsCache.value[key] = setting.value;
      }
    } catch {
      // 忽略错误
    }
  }

  /** 刷新所有设置缓存 */
  async function refreshAllSettings(): Promise<void> {
    try {
      const allSettings = await invoke<Setting[]>("get_settings");
      allSettings.forEach((s) => {
        if (s.value) {
          settingsCache.value[s.key] = s.value;
        }
      });
    } catch {
      // 忽略错误
    }
  }

  return {
    settingsCache,
    getSetting,
    getSettingNumber,
    setSetting,
    refreshSetting,
    refreshAllSettings,
  };
});
