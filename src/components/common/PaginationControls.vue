<!--
  PaginationControls.vue - 分页控件组件

  功能：
  - 首页/上一页/页码输入/下一页/末页按钮
  - 显示总条数
-->
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Home, ChevronLeft, ChevronRight, SkipForward } from "@lucide/vue";
import type { FooterState } from "../../composables/footer";

const props = defineProps<{
  footer: FooterState;
}>();

const totalPages = computed(() => Math.ceil(props.footer.totalRecords / props.footer.pageSize) || 1);
const jumpInput = ref(String(props.footer.currentPage));
let jumpTimer: ReturnType<typeof setTimeout> | null = null;

function goTo(page: number) {
  if (page < 1 || page > totalPages.value) return;
  props.footer.currentPage = page;
  if (props.footer.onPageChange) props.footer.onPageChange(page);
}

function onJumpInput() {
  if (jumpTimer) clearTimeout(jumpTimer);
  jumpTimer = setTimeout(() => {
    const p = parseInt(jumpInput.value, 10);
    if (!isNaN(p)) goTo(p);
  }, 500);
}

watch(() => props.footer.currentPage, (p) => {
  jumpInput.value = String(p);
});
</script>

<template>
  <div class="pagination-controls">
    <button class="btn-icon btn-icon-info" :disabled="footer.currentPage <= 1" @click="goTo(1)" title="首页">
      <Home :size="16" />
    </button>
    <button class="btn-icon btn-icon-info" :disabled="footer.currentPage <= 1" @click="goTo(footer.currentPage - 1)" title="上一页">
      <ChevronLeft :size="16" />
    </button>

    <span class="btf-page-info">
      <input v-model="jumpInput" class="btf-input" @input="onJumpInput" />
      <span class="btf-text">/ {{ totalPages }} 页</span>
    </span>

    <button class="btn-icon btn-icon-info" :disabled="footer.currentPage >= totalPages" @click="goTo(footer.currentPage + 1)" title="下一页">
      <ChevronRight :size="16" />
    </button>
    <button class="btn-icon btn-icon-info" :disabled="footer.currentPage >= totalPages" @click="goTo(totalPages)" title="末页">
      <SkipForward :size="16" />
    </button>

    <span class="btf-text">共 {{ footer.totalRecords }} 条</span>
  </div>
</template>

<style scoped>
.pagination-controls {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.btf-page-info {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.btf-input {
  width: 48px;
  padding: 0.125rem 0.25rem;
  border-radius: 4px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8125rem;
  text-align: center;
}

.btf-text {
  color: var(--text-secondary);
  white-space: nowrap;
}
</style>