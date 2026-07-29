<!--
  StandardizedTablePagination.vue - 表格分页组件

  功能：
  - 显示总记录数和当前页码
  - 提供页码导航按钮
  - 支持上一页/下一页
  - 支持页码省略显示
-->
<script setup lang="ts">

const props = defineProps<{
  currentPage: number;
  totalPages: number;
  totalRecords: number;
}>();

const emit = defineEmits<{
  "page-change": [page: number];
}>();

function goToPage(page: number) {
  if (page < 1 || page > props.totalPages) return;
  emit("page-change", page);
}

function getPageNumbers(): (number | string)[] {
  const total = props.totalPages;
  const current = props.currentPage;
  const pages: (number | string)[] = [];

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i);
  } else {
    pages.push(1, 2, 3);
    if (current > 5) pages.push("...");
    for (
      let i = Math.max(4, current - 1);
      i <= Math.min(total - 2, current + 1);
      i++
    ) {
      pages.push(i);
    }
    if (current < total - 4) pages.push("...");
    pages.push(total - 2, total - 1, total);
  }

  return pages;
}
</script>

<template>
  <div class="table-pagination">
    <span class="pagination-info">
      共 {{ totalRecords }} 条记录，第 {{ currentPage }}/{{ totalPages }} 页
    </span>
    <div class="pagination-controls">
      <button
        class="page-btn"
        :disabled="currentPage <= 1"
        @click="goToPage(currentPage - 1)"
      >
        ‹
      </button>
      <template v-for="page in getPageNumbers()" :key="page">
        <button
          v-if="typeof page === 'number'"
          class="page-btn"
          :class="{ active: page === currentPage }"
          @click="goToPage(page)"
        >
          {{ page }}
        </button>
        <span v-else class="page-ellipsis">{{ page }}</span>
      </template>
      <button
        class="page-btn"
        :disabled="currentPage >= totalPages"
        @click="goToPage(currentPage + 1)"
      >
        ›
      </button>
    </div>
  </div>
</template>

<style scoped>
.table-pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.pagination-info {
  white-space: nowrap;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.page-btn {
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.page-btn:hover:not(:disabled):not(.active) {
  background: var(--bg-hover);
  border-color: var(--accent);
}

.page-btn.active {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.page-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-ellipsis {
  padding: 0 0.375rem;
  color: var(--text-muted);
}

@media (max-width: 768px) {
  .table-pagination {
    flex-direction: column;
    gap: 0.5rem;
    align-items: center;
  }
}
</style>