<!--
  PackageRowActions.vue - 软件包表格行操作按钮组

  功能：
  - 查看详情、编辑
  - 从 AUR / PKGBUILD 同步
  - 更新上游信息
  - 删除
-->
<script setup lang="ts">
import { Icon } from "../../icons";

defineProps<{
  pkgname: string;
  /** 判断指定行的某个操作是否正在执行中 */
  isRowLoading: (pkgname: string, action: string) => boolean;
}>();

const emit = defineEmits<{
  view: [pkgname: string];
  edit: [pkgname: string];
  "sync-aur": [pkgname: string];
  "sync-pkgbuild": [pkgname: string];
  "check-upstream": [pkgname: string];
  delete: [pkgname: string];
}>();
</script>

<template>
  <button
    class="btn-icon btn-icon-info"
    @click.stop="emit('view', pkgname)"
    title="查看详情"
  >
    <component :is="Icon.actionInfo" :size="14" />
  </button>
  <button
    class="btn-icon btn-icon-warning"
    @click.stop="emit('edit', pkgname)"
    title="软件编辑"
  >
    <component :is="Icon.actionEdit" :size="14" />
  </button>
  <button
    class="btn-icon btn-icon-accent"
    @click.stop="emit('sync-aur', pkgname)"
    :disabled="isRowLoading(pkgname, 'sync-aur')"
    title="从AUR同步"
  >
    <component :is="Icon.syncAur" :size="14" />
  </button>
  <button
    class="btn-icon btn-icon-accent"
    @click.stop="emit('sync-pkgbuild', pkgname)"
    :disabled="isRowLoading(pkgname, 'sync-pkgbuild')"
    title="从PKGBUILD同步"
  >
    <component :is="Icon.syncPkgbuild" :size="14" />
  </button>
  <button
    class="btn-icon btn-icon-info"
    @click.stop="emit('check-upstream', pkgname)"
    :disabled="isRowLoading(pkgname, 'check-upstream')"
    title="更新上游信息"
  >
    <component :is="Icon.actionSearch" :size="14" />
  </button>
  <button
    class="btn-icon btn-icon-danger"
    @click.stop="emit('delete', pkgname)"
    :disabled="isRowLoading(pkgname, 'delete')"
    title="删除"
  >
    <component :is="Icon.actionDelete" :size="14" />
  </button>
</template>
