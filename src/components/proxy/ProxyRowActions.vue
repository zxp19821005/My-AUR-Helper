<!--
  ProxyRowActions.vue - 代理表格行操作按钮组

  功能：
  - 启用/禁用代理
  - 测试代理
  - 删除代理

  操作列统一使用带语义背景色的 btn-icon 样式：
  - 启用/禁用（toggle）：蓝色 info
  - 测试（test）：绿色 success
  - 删除（delete）：红色 danger
-->
<script setup lang="ts">
import { Icon } from "../../icons";

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
  <button
    class="btn-icon btn-icon-info"
    @click.stop="emit('toggle', row)"
    :title="row.is_active ? '禁用' : '启用'"
  >
    <component :is="Icon.sourceAur" :size="14" />
  </button>

  <button
    class="btn-icon btn-icon-success"
    @click.stop="row.proxy_id !== null && emit('test', row.proxy_id)"
    :disabled="row.proxy_id !== null && testingIds.has(row.proxy_id)"
    title="测试代理"
  >
    <component :is="Icon.testSingle" :size="14" />
  </button>

  <button
    class="btn-icon btn-icon-danger"
    @click.stop="emit('delete', row)"
    title="删除"
  >
    <component :is="Icon.actionDelete" :size="14" />
  </button>
</template>
