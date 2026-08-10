<!--
  LogToolbar.vue - 日志查看器顶部工具栏

  功能：级别筛选下拉、搜索框、清空当天日志按钮。
  交互：级别/搜索使用 v-model 受控，清空日志 emit clear-logs 由父组件处理（含确认与调用）。
-->
<script setup lang="ts">
import { Trash2, X } from "@lucide/vue";

const levelFilter = defineModel<string>("levelFilter", { default: "" });
const searchQuery = defineModel<string>("searchQuery", { default: "" });
defineProps<{ loading: boolean }>();
const emit = defineEmits<{
  clearLogs: [];
}>();
</script>

<template>
  <div class="log-toolbar">
    <div class="log-toolbar-left">
      <div class="level-filter-wrapper">
        <select v-model="levelFilter" class="level-select">
          <option value="">全部级别</option>
          <option value="INFO">INFO</option>
          <option value="WARN">WARN</option>
          <option value="ERROR">ERROR</option>
          <option value="DEBUG">DEBUG</option>
        </select>
        <button
          v-if="levelFilter"
          class="btn-icon btn-icon-sm btn-icon-secondary"
          @click="levelFilter = ''"
          title="清除筛选"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
    <div class="log-toolbar-right">
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="搜索日志..."
        />
      </div>
      <button
        class="btn-icon btn-icon-danger"
        :disabled="loading"
        @click="emit('clearLogs')"
        title="清空当天日志"
      >
        <Trash2 :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.log-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
  min-height: 44px;
}
.log-toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.log-toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
.level-filter-wrapper {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
.level-select {
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: border-color 0.15s;
}
.level-select:hover {
  border-color: var(--accent);
}
.level-select:focus {
  outline: none;
  border-color: var(--accent);
}
.search-box {
  display: flex;
  align-items: center;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  transition: border-color 0.15s;
}
.search-box:focus-within {
  border-color: var(--accent);
}
.search-input {
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  width: 140px;
}
.search-input::placeholder {
  color: var(--text-muted);
}
</style>
