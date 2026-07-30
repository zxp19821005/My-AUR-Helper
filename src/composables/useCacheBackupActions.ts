/**
 * useCacheBackupActions.ts - 缓存包备份操作 composable
 *
 * 功能：
 * - 备份目录去重（deduplicate_backups）
 * - 备份新版到已有目录（backup_cache_to_existing）
 * - 备份到子目录弹窗的打开与结果处理
 *
 * 使用场景：CacheManager.vue 页面
 */
import { ref, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { addMessage, type FooterState } from "./footer";
import type { DeduplicateResult } from "../types";

/**
 * 创建缓存备份操作集合
 *
 * @param footer 底部状态栏状态（用于消息提示）
 * @param backupPath 备份目录路径
 * @param selectedIds 选中行索引集合
 * @param selectedFilenames 选中的缓存文件名列表
 * @param loading 页面加载状态
 * @returns 备份操作函数和弹窗显示状态
 */
export function useCacheBackupActions(
  footer: FooterState,
  backupPath: Ref<string>,
  selectedIds: Ref<Set<number>>,
  selectedFilenames: Ref<string[]>,
  loading: Ref<boolean>
) {
  /** 是否显示"备份到"弹窗 */
  const showBackupToModal = ref(false);

  /** 对备份目录执行去重，删除旧版本文件 */
  async function handleDedup() {
    if (!backupPath.value) {
      addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
      return;
    }
    if (!confirm("确定要对备份目录进行去重吗？将删除旧版本文件。")) return;
    loading.value = true;
    try {
      const result = await invoke<DeduplicateResult>("deduplicate_backups", {
        backupPath: backupPath.value,
      });
      const msg = `去重完成：删除 ${result.removed_files} 个文件，${result.removed_records} 条记录`;
      if (result.errors.length > 0) {
        addMessage(footer, "warning", `${msg}，错误: ${result.errors.join("; ")}`);
      } else {
        addMessage(footer, "success", msg);
      }
    } catch (e) {
      addMessage(footer, "error", `去重失败: ${e}`);
    } finally {
      loading.value = false;
    }
  }

  /** 将选中的缓存包备份到已有备份目录 */
  async function handleBackupNewVersion() {
    if (selectedIds.value.size === 0) {
      addMessage(footer, "warning", "请先选择要备份的缓存包");
      return;
    }
    if (!backupPath.value) {
      addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
      return;
    }
    loading.value = true;
    try {
      const [success, errors] = await invoke<[number, string[]]>(
        "backup_cache_to_existing",
        {
          filenames: selectedFilenames.value,
          backupPath: backupPath.value,
        }
      );
      notifyBackupResult(success, errors);
      selectedIds.value.clear();
    } catch (e) {
      addMessage(footer, "error", `备份失败: ${e}`);
    } finally {
      loading.value = false;
    }
  }

  /** 打开"备份到子目录"弹窗（校验选中项和备份目录） */
  function openBackupToModal() {
    if (selectedIds.value.size === 0) {
      addMessage(footer, "warning", "请先选择要备份的缓存包");
      return;
    }
    if (!backupPath.value) {
      addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
      return;
    }
    showBackupToModal.value = true;
  }

  /** 处理"备份到"弹窗的备份结果 */
  function handleBackupSuccess(result: [number, string[]]) {
    const [success, errors] = result;
    notifyBackupResult(success, errors);
    selectedIds.value.clear();
  }

  /** 根据备份结果发送成功/警告消息 */
  function notifyBackupResult(success: number, errors: string[]) {
    const msg =
      `备份完成：成功 ${success} 个` +
      (errors.length > 0 ? `，错误: ${errors.join("; ")}` : "");
    addMessage(footer, errors.length > 0 ? "warning" : "success", msg);
  }

  return {
    showBackupToModal,
    handleDedup,
    handleBackupNewVersion,
    openBackupToModal,
    handleBackupSuccess,
  };
}
