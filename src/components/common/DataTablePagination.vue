<!--
  DataTablePagination.vue - 分页控件组件

  功能：
  - 显示分页信息（共X条，第X/Y页）
  - 显示页码列表（支持省略号）
  - 上一页/下一页按钮
  - 点击页码跳转
-->
<script setup lang="ts">
import { computed } from "vue";

interface Props {
  currentPage: number;
  totalRecords: number;
  totalPages: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "goToPage", page: number): void;
}>();

/** 页码列表（用于分页控件） */
const pageNumbers = computed(() => {
  const total = props.totalPages;
  const current = props.currentPage;
  const pages: (number | string)[] = [];

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i);
  } else {
    pages.push(1, 2, 3);
    if (current > 5) pages.push("...");
    for (let i = Math.max(4, current - 1); i <= Math.min(total - 2, current + 1); i++) {
      pages.push(i);
    }
    if (current < total - 4) pages.push("...");
    pages.push(total - 2, total - 1, total);
  }

  return pages;
});
</script>

<template>
  <div v-if="totalRecords > 0" class="pagination-bar">
    <span class="pagination-info">共 {{ totalRecords }} 条记录，第 {{ currentPage }}/{{ totalPages }} 页</span>
    <div class="pagination-controls">
      <button class="page-btn" :disabled="currentPage <= 1" @click="emit('goToPage', currentPage - 1)">&laquo;</button>
      <template v-for="page in pageNumbers" :key="page">
        <button v-if="typeof page === 'number'" class="page-btn" :class="{ active: page === currentPage }" @click="emit('goToPage', page)">{{ page }}</button>
        <span v-else class="page-ellipsis">{{ page }}</span>
      </template>
      <button class="page-btn" :disabled="currentPage >= totalPages" @click="emit('goToPage', currentPage + 1)">&raquo;</button>
    </div>
  </div>
</template>

<style scoped>
.pagination-bar { display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0; font-size: 0.875rem; color: var(--text-secondary); }
.pagination-controls { display: flex; align-items: center; gap: 0.25rem; }
.page-btn { min-width: 2rem; height: 2rem; padding: 0 0.5rem; border: 1px solid var(--border); border-radius: 4px; background: var(--bg-primary); color: var(--text-primary); font-size: 0.875rem; cursor: pointer; transition: all 0.15s; }
.page-btn:hover:not(:disabled):not(.active) { background: var(--hover-bg, rgba(128, 128, 128, 0.1)); }
.page-btn.active { background: var(--accent, #6c63ff); color: white; border-color: var(--accent, #6c63ff); }
.page-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.page-ellipsis { padding: 0 0.25rem; color: var(--text-secondary); }
</style>
