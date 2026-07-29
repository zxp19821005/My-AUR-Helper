<!--
  BackupToolbar.vue - 备份管理工具栏

  功能：
  - 搜索输入框
  - 子目录筛选下拉框
  - 操作按钮：清空、扫描、去重、批量安装、删除选中
-->
<script setup lang="ts">
import { Trash2, Scan, Copy, Download } from "@lucide/vue";
import StandardizedButton from "./base/StandardizedButton.vue";
import StandardizedSelect from "./base/StandardizedSelect.vue";

defineProps<{
  searchQuery: string;
  subdirectoryFilter: string;
  subdirectories: string[];
  loading: boolean;
  scanning: boolean;
  selectedCount: number;
  installing: boolean;
}>();

const emit = defineEmits<{
  "update:searchQuery": [value: string];
  "update:subdirectoryFilter": [value: string];
  "clear-table": [];
  "scan-directory": [];
  deduplicate: [];
  "batch-install": [];
  "delete-selected": [];
}>();
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <input
        type="text"
        :value="searchQuery"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        placeholder="搜索备份包..."
        class="search-input"
      />
    </div>
    <div class="toolbar-right">
      <StandardizedSelect
        :modelValue="subdirectoryFilter"
        @update:modelValue="emit('update:subdirectoryFilter', String($event))"
        size="sm"
      >
        <option value="">全部子目录</option>
        <option v-for="dir in subdirectories" :key="dir" :value="dir">{{ dir }}</option>
      </StandardizedSelect>

      <StandardizedButton
        variant="danger"
        size="sm"
        :loading="loading"
        @click="emit('clear-table')"
        title="清空备份表"
      >
        <Trash2 :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading || scanning"
        @click="emit('scan-directory')"
        title="扫描备份目录"
      >
        <Scan :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="secondary"
        size="sm"
        :loading="loading"
        @click="emit('deduplicate')"
        title="软件去重"
      >
        <Copy :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="primary"
        size="sm"
        :disabled="selectedCount === 0 || installing"
        @click="emit('batch-install')"
        title="批量安装备份包"
      >
        <Download :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="danger"
        size="sm"
        :disabled="selectedCount === 0"
        @click="emit('delete-selected')"
        title="删除选中"
      >
        <Trash2 :size="16" />
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