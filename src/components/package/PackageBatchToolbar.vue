<!--
  PackageBatchToolbar.vue - 软件包列表页「批量操作按钮组」

  功能：渲染工具栏左侧 8 个彩色图标按钮（从 AUR 同步 / 按维护者批量同步 /
        AUR 导入新包 / 从 PKGBUILD 同步 / 添加软件 / 更新 AUR 信息 /
        检查上游 / 删除选中），并根据加载态与选中数量控制禁用态。
  工作流程：纯展示组件，父组件通过 props 传入 loading / checkLoading / selectedCount；
        点击按钮仅 emit 语义事件，选中集合与具体 IPC 调用仍由父组件持有，
        组件本身不 import 任何 composable / store，保证与后续重命名彻底解耦。
  适用场景：PageToolbar 默认插槽（.toolbar-left）内的批量操作区；多个根节点平铺，
        保持原有 flex 布局与父级插槽作用域样式完全一致。
-->
<script setup lang="ts">
import { Icon } from "@/icons";

const props = defineProps<{
  /** 同步 / 检查类操作进行中：禁用多数批量按钮，防止重复提交 */
  loading: boolean;
  /** 上游检查进行中：与 loading 共同禁用「检查上游」按钮 */
  checkLoading: boolean;
  /** 当前选中包数量，为 0 时禁用「删除选中」按钮 */
  selectedCount: number;
}>();

const emit = defineEmits<{
  (e: "sync-aur"): void;
  (e: "sync-by-maintainer"): void;
  (e: "open-aur-import"): void;
  (e: "sync-pkgbuild"): void;
  (e: "add"): void;
  (e: "update-aur"): void;
  (e: "check-upstream"): void;
  (e: "delete"): void;
}>();
</script>

<template>
  <button class="btn-icon btn-color-aur" @click="emit('sync-aur')" :disabled="props.loading" title="从AUR同步">
    <component :is="Icon.syncAur" :size="16" />
  </button>
  <button class="btn-icon btn-color-import" @click="emit('sync-by-maintainer')" :disabled="props.loading" title="按维护者批量同步（补齐AUR上维护但本地缺失的包，保留已配置的上游地址）">
    <component :is="Icon.actionDownload" :size="16" />
  </button>
  <button class="btn-icon btn-color-import" @click="emit('open-aur-import')" :disabled="props.loading" title="从AUR导入新包">
    <component :is="Icon.actionDownload" :size="16" />
  </button>
  <button class="btn-icon btn-color-pkgbuild" @click="emit('sync-pkgbuild')" :disabled="props.loading" title="从PKGBUILD同步">
    <component :is="Icon.syncPkgbuild" :size="16" />
  </button>
  <button class="btn-icon btn-color-add" @click="emit('add')" title="添加软件">
    <component :is="Icon.actionAdd" :size="16" />
  </button>
  <button class="btn-icon btn-color-update" @click="emit('update-aur')" :disabled="props.loading" title="更新AUR信息">
    <component :is="Icon.actionRefresh" :size="16" />
  </button>
  <button class="btn-icon btn-color-check" @click="emit('check-upstream')" :disabled="props.checkLoading || props.loading" title="检查上游">
    <component :is="Icon.actionSearch" :size="16" />
  </button>
  <button class="btn-icon btn-color-delete" @click="emit('delete')" :disabled="props.selectedCount === 0" title="删除选中">
    <component :is="Icon.actionDelete" :size="16" />
  </button>
</template>

<style scoped>
/*
 * 本区块样式原位于 PackageList.css（scoped，作用域为 PackageList.vue）。
 * 抽为子组件后插槽元素改为子组件作用域，故在此自带等价 scoped 样式以保持视觉一致。
 * .toolbar-filter-select 由全局 filter-styles.css 提供，无需重复声明。
 */
.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.375rem;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-icon:hover:not(:disabled) {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* 彩色操作按钮 - 每个按钮使用独特颜色 */
.btn-color-aur {
  color: #1793d1; /* AUR 蓝 */
}
.btn-color-aur:hover:not(:disabled) {
  background-color: rgba(23, 147, 209, 0.1);
  color: #1793d1;
}

.btn-color-pkgbuild {
  color: #f59e0b; /* PKGBUILD 橙 */
}
.btn-color-pkgbuild:hover:not(:disabled) {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.btn-color-add {
  color: #10b981; /* 添加 绿 */
}
.btn-color-add:hover:not(:disabled) {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.btn-color-update {
  color: #3b82f6; /* 更新 蓝 */
}
.btn-color-update:hover:not(:disabled) {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.btn-color-check {
  color: #8b5cf6; /* 检查 紫 */
}
.btn-color-check:hover:not(:disabled) {
  background-color: rgba(139, 92, 246, 0.1);
  color: #8b5cf6;
}

.btn-color-delete {
  color: #ef4444; /* 删除 红 */
}
.btn-color-delete:hover:not(:disabled) {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.btn-color-import {
  color: #0ea5e9; /* 导入 青色 */
}
.btn-color-import:hover:not(:disabled) {
  background-color: rgba(14, 165, 233, 0.1);
  color: #0ea5e9;
}
</style>
