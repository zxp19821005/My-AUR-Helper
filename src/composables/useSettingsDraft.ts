/**
 * useSettingsDraft.ts - 设置草稿模型
 *
 * 功能：
 * - 统一管理"先编辑、后保存"的设置交互
 * - draft 为可编辑副本，saved 为上次持久化快照
 * - dirty 标识是否有未保存修改
 * - reset 撤销未保存修改（恢复到 saved）
 * - commit 在持久化成功后调用，将 saved 同步为当前 draft
 *
 * 适用：所有设置页（表单类、缓存目录列表、外观等），
 * 重置语义统一为"撤销到上次已保存的值"。
 */
import { ref, computed, type Ref, type ComputedRef } from "vue";

/** 深拷贝（设置数据均为纯字符串/普通对象，JSON 克隆足够且安全） */
function deepClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export interface SettingsDraft<T> {
  /** 可编辑副本，绑定到表单/列表 */
  draft: Ref<T>;
  /** 上次持久化快照，用于 reset 与 dirty 比对 */
  saved: Ref<T>;
  /** 是否正在保存（用于禁用按钮/loading） */
  saving: Ref<boolean>;
  /** 是否存在未保存修改 */
  dirty: ComputedRef<boolean>;
  /** 放弃未保存修改，恢复到 saved */
  reset: () => void;
  /** 持久化成功后调用，将 saved 同步为当前 draft */
  commit: () => void;
}

/**
 * 创建设置草稿模型
 * @param initial 初始值（通常为从数据库/本地存储加载的快照）
 */
export function useSettingsDraft<T>(initial: T): SettingsDraft<T> {
  const saved = ref(deepClone(initial)) as Ref<T>;
  const draft = ref(deepClone(initial)) as Ref<T>;
  const saving = ref(false);
  const dirty = computed(
    () => JSON.stringify(draft.value) !== JSON.stringify(saved.value),
  );

  function reset(): void {
    draft.value = deepClone(saved.value);
  }

  function commit(): void {
    saved.value = deepClone(draft.value);
  }

  return { draft, saved, saving, dirty, reset, commit };
}
