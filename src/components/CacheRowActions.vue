<!--
  CacheRowActions.vue - 缓存表格行操作按钮组

  功能：
  - 删除缓存文件
-->
<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import StandardizedButton from "./base/StandardizedButton.vue";

defineProps<{
  loading: boolean;
}>();

const emit = defineEmits<{
  delete: [];
}>();
</script>

<template>
  <StandardizedButton
    variant="danger"
    size="sm"
    :loading="loading"
    @click.stop="emit('delete')"
    title="删除"
  >
    <Trash2 :size="14" />
  </StandardizedButton>
</template>