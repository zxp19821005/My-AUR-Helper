<!--
  ProxyRowActions.vue - 代理表格行操作按钮组

  功能：
  - 启用/禁用代理
  - 测试代理
  - 删除代理
-->
<script setup lang="ts">
import { Globe, Zap, Trash2 } from "@lucide/vue";
import StandardizedButton from "./base/StandardizedButton.vue";

defineProps<{
  row: any;
  testingIds: Set<number>;
}>();

const emit = defineEmits<{
  toggle: [row: any];
  test: [proxyId: number];
  delete: [row: any];
}>();
</script>

<template>
  <StandardizedButton
    variant="outline"
    size="sm"
    @click.stop="emit('toggle', row)"
    :title="row.is_active ? '禁用' : '启用'"
  >
    <Globe :size="14" />
  </StandardizedButton>

  <StandardizedButton
    variant="outline"
    size="sm"
    @click.stop="row.proxy_id !== null && emit('test', row.proxy_id)"
    :disabled="row.proxy_id !== null && testingIds.has(row.proxy_id)"
    title="测试代理"
  >
    <Zap :size="14" />
  </StandardizedButton>

  <StandardizedButton
    variant="danger"
    size="sm"
    @click.stop="emit('delete', row)"
    title="删除"
  >
    <Trash2 :size="14" />
  </StandardizedButton>
</template>