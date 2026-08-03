<!--
  CacheToolbar.vue - 缓存管理工具栏

  功能：
  - 搜索输入框
  - 缓存目录筛选下拉框
  - 操作按钮：扫描、清空、删除选中、去重、备份新版、备份到
-->
<script setup lang="ts">
import { Trash2, Scan, Copy, GitBranch } from "@lucide/vue";
import StandardizedButton from "../base/StandardizedButton.vue";
import StandardizedSelect from "../base/StandardizedSelect.vue";

defineProps<{
  searchQuery: string;
  sourceDirFilter: string;
  sourceDirs: { name: string; path: string }[];
  loading: boolean;
  scanning: boolean;
  selectedCount: number;
}>();

const emit = defineEmits<{
  "update:searchQuery": [value: string];
  "update:sourceDirFilter": [value: string];
  scan: [];
  "clear-table": [];
  "delete-selected": [];
  dedup: [];
  "backup-new-version": [];
  "backup-to": [];
}>();
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <input
        type="text"
        :value="searchQuery"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        placeholder="搜索缓存包..."
        class="search-input"
      />
    </div>
    <div class="toolbar-right">
      <StandardizedSelect
        :modelValue="sourceDirFilter"
        @update:modelValue="emit('update:sourceDirFilter', String($event))"
        size="sm"
      >
        <option value="">全部缓存目录</option>
        <option v-for="dir in sourceDirs" :key="dir.name" :value="dir.name">
          {{ dir.name }}
        </option>
      </StandardizedSelect>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading || scanning"
        @click="emit('scan')"
        title="扫描所有缓存目录"
      >
        <Scan :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="danger"
        size="sm"
        :loading="loading"
        @click="emit('clear-table')"
        title="清空缓存表"
      >
        <Trash2 :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :disabled="selectedCount === 0"
        @click="emit('delete-selected')"
        title="删除选中"
      >
        <Trash2 :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading"
        @click="emit('dedup')"
        title="去重（保留最新版本）"
      >
        <GitBranch :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading"
        @click="emit('backup-new-version')"
        title="备份新版（自动比较版本，将更新的包备份到已有位置）"
      >
        <Copy :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading"
        :disabled="selectedCount === 0"
        @click="emit('backup-to')"
        title="备份到（选择子目录）"
      >
        <Copy :size="16" />
      </StandardizedButton>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.toolbar-left {
  flex: 1;
  min-width: 200px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.search-input {
  width: 100%;
  max-width: 320px;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.search-input:focus {
  border-color: var(--accent);
  outline: none;
}
</style>