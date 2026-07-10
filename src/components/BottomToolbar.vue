<script setup lang="ts">
import { ref, computed, inject, watch } from "vue";
import { FOOTER_KEY } from "../composables/footer";
import { ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight } from "@lucide/vue";

const footer = inject(FOOTER_KEY)!;

const totalPages = computed(() => Math.ceil(footer.totalRecords / footer.pageSize) || 1);

const jumpInput = ref(String(footer.currentPage));
let jumpTimer: ReturnType<typeof setTimeout> | null = null;

function goTo(page: number) {
  if (page < 1 || page > totalPages.value) return;
  footer.currentPage = page;
  if (footer.onPageChange) footer.onPageChange(page);
}

function onJumpInput() {
  if (jumpTimer) clearTimeout(jumpTimer);
  jumpTimer = setTimeout(() => {
    const p = parseInt(jumpInput.value, 10);
    if (!isNaN(p)) goTo(p);
  }, 500);
}

watch(() => footer.currentPage, (p) => {
  jumpInput.value = String(p);
});
</script>

<template>
  <div class="bottom-toolbar">
    <div class="btf-left">
      <span v-if="footer.infoText">{{ footer.infoText }}</span>
    </div>
    <div class="btf-center">
      <template v-if="footer.showPagination">
        <button class="btn-icon btn-icon-accent" :disabled="footer.currentPage <= 1" @click="goTo(1)" title="首页">
          <ChevronsLeft :size="16" />
        </button>
        <button class="btn-icon btn-icon-accent" :disabled="footer.currentPage <= 1" @click="goTo(footer.currentPage - 1)" title="上一页">
          <ChevronLeft :size="16" />
        </button>

        <span class="btf-page-info">
          <input v-model="jumpInput" class="btf-input" @input="onJumpInput" />
          <span class="btf-text">/ {{ totalPages }} 页</span>
        </span>

        <button class="btn-icon btn-icon-accent" :disabled="footer.currentPage >= totalPages" @click="goTo(footer.currentPage + 1)" title="下一页">
          <ChevronRight :size="16" />
        </button>
        <button class="btn-icon btn-icon-accent" :disabled="footer.currentPage >= totalPages" @click="goTo(totalPages)" title="末页">
          <ChevronsRight :size="16" />
        </button>

        <span class="btf-text">共 {{ footer.totalRecords }} 条</span>
      </template>
    </div>
    <div class="btf-right">
      <div v-if="footer.progress" class="btf-progress">
        <span v-if="footer.progress.message" class="btf-progress-msg">{{ footer.progress.message }}</span>
        <div class="btf-progress-track">
          <div class="btf-progress-fill" :style="{ width: (footer.progress.current / footer.progress.total * 100) + '%' }"></div>
        </div>
        <span class="btf-text">{{ footer.progress.current }} / {{ footer.progress.total }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 1.25rem;
  border-top: 1px solid var(--border);
  background-color: var(--bg-secondary);
  min-height: 36px;
  font-size: 0.8125rem;
  gap: 1rem;
}
.btf-left {
  flex: 1;
  text-align: left;
  color: var(--text-secondary);
}
.btf-center {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}
.btf-right {
  flex: 1;
  display: flex;
  justify-content: flex-end;
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
.btf-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.btf-progress-msg {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
  font-size: 0.75rem;
}
.btf-progress-track {
  width: 120px;
  height: 6px;
  border-radius: 3px;
  background-color: var(--border);
  overflow: hidden;
}
.btf-progress-fill {
  height: 100%;
  border-radius: 3px;
  background-color: var(--accent);
  transition: width 0.2s;
}
</style>
