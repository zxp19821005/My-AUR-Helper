<!--
  CacheRowActions.vue - 缓存表格行操作按钮组

  功能：
  - 删除缓存文件

  操作列统一使用带语义背景色的 btn-icon 样式：
  - 删除（delete）：红色 danger
-->
<script setup lang="ts">
import { Icon } from "../../icons";

defineProps<{
  loading: boolean;
}>();

const emit = defineEmits<{
  delete: [];
}>();
</script>

<template>
  <button
    class="btn-icon btn-icon-danger"
    :disabled="loading"
    @click.stop="emit('delete')"
    title="删除"
  >
    <component :is="Icon.actionDelete" :size="14" />
  </button>
</template>
