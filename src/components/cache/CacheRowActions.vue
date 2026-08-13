<!--
  CacheRowActions.vue - 缓存表格行操作按钮组

  功能：
  - 查看包信息
  - 安装缓存包
  - 删除缓存文件

  操作列统一使用带语义背景色的 btn-icon 样式：
  - 查看信息（view-info）：蓝色 info
  - 安装（install）：橙色 warning
  - 删除（delete）：红色 danger
-->
<script setup lang="ts">
import { Icon } from "../../icons";

defineProps<{
  row: any;
  loading: boolean;
  installing: boolean;
}>();

const emit = defineEmits<{
  "view-info": [row: any];
  install: [row: any];
  delete: [row: any];
}>();
</script>

<template>
  <button
    class="btn-icon btn-icon-info"
    @click.stop="emit('view-info', row)"
    title="查看信息"
  >
    <component :is="Icon.actionInfo" :size="14" />
  </button>

  <button
    class="btn-icon btn-icon-warning"
    @click.stop="emit('install', row)"
    :disabled="installing"
    title="安装"
  >
    <component :is="Icon.install" :size="14" />
  </button>

  <button
    class="btn-icon btn-icon-danger"
    @click.stop="emit('delete', row)"
    :disabled="loading"
    title="删除"
  >
    <component :is="Icon.actionDelete" :size="14" />
  </button>
</template>
