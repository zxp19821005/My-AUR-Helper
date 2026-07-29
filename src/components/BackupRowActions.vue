<!--
  BackupRowActions.vue - 备份表格行操作按钮组

  功能：
  - 查看包信息
  - 安装备份包
  - 删除备份
-->
<script setup lang="ts">
import { Info, Download, Trash2 } from "@lucide/vue";
import StandardizedButton from "./base/StandardizedButton.vue";

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
  <StandardizedButton
    variant="outline"
    size="sm"
    @click.stop="emit('view-info', row)"
    title="查看信息"
  >
    <Info :size="14" />
  </StandardizedButton>

  <StandardizedButton
    variant="primary"
    size="sm"
    @click.stop="emit('install', row)"
    :disabled="installing"
    title="安装"
  >
    <Download :size="14" />
  </StandardizedButton>

  <StandardizedButton
    variant="danger"
    size="sm"
    @click.stop="emit('delete', row)"
    :disabled="loading"
    title="删除"
  >
    <Trash2 :size="14" />
  </StandardizedButton>
</template>