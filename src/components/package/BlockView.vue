<!--
  BlockView.vue - 软件包块视图组件

  功能：
  - 以卡片形式展示软件包列表
  - 支持选中、查看详情、编辑、同步、检查、删除等操作
  - 响应式网格布局

  参考：Clash-verge 连接页面风格
-->
<script setup lang="ts">
import { computed } from "vue";
import type { SoftwareListEntry } from "@/types";
import { fmtTimestamp } from "@/composables/usePackageList";
import { Icon } from "@/icons";

interface Props {
  entries: SoftwareListEntry[];
  selectedPkgnames: Set<string>;
  isRowLoading?: (pkgname: string, action?: string) => boolean;
  pageSize?: number;
  currentPage?: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "select", pkgname: string): void;
  (e: "select-all"): void;
  (e: "view", pkgname: string): void;
  (e: "edit", pkgname: string): void;
  (e: "sync-aur", pkgname: string): void;
  (e: "sync-pkgbuild", pkgname: string): void;
  (e: "check-upstream", pkgname: string): void;
  (e: "delete", pkgname: string): void;
  (e: "page-change", page: number): void;
}>();

const isSelected = computed(() => (pkgname: string) =>
  props.selectedPkgnames.has(pkgname)
);

const totalPages = computed(() =>
  Math.ceil(props.entries.length / (props.pageSize || 50))
);

function getPageEntries() {
  const start = (props.currentPage || 1) - 1;
  const end = start + (props.pageSize || 50);
  return props.entries.slice(start, end);
}
</script>

<template>
  <div class="block-view">
    <!-- 全选控制 -->
    <div class="block-header">
      <label class="select-all-label">
        <input
          type="checkbox"
          :checked="entries.length > 0 && entries.every(e => selectedPkgnames.has(e.pkgname))"
          @change="emit('select-all')"
        />
        <span>全选</span>
      </label>
      <span class="block-count">共 {{ entries.length }} 个软件包</span>
    </div>

    <!-- 软件包卡片网格 -->
    <div class="block-grid">
      <div
        v-for="entry in getPageEntries()"
        :key="entry.pkgname"
        class="block-card"
        :class="{ 'card-selected': isSelected(entry.pkgname), 'card-outdated': entry.is_outdated }"
        @click="emit('select', entry.pkgname)"
      >
        <!-- 卡片头部 -->
        <div class="card-header">
          <div class="card-title">
            <input
              type="checkbox"
              :checked="isSelected(entry.pkgname)"
              @click.stop
              @change="emit('select', entry.pkgname)"
            />
            <span class="pkg-name">{{ entry.pkgname }}</span>
          </div>
          <span
            class="status-badge"
            :class="entry.is_outdated ? 'badge-outdated' : 'badge-up-to-date'"
          >
            {{ entry.is_outdated ? '需更新' : '最新' }}
          </span>
        </div>

        <!-- 卡片内容 -->
        <div class="card-body">
          <div class="info-row">
            <span class="label">AUR版本:</span>
            <span class="value">{{ entry.aur_version || '-' }}</span>
          </div>
          <div class="info-row">
            <span class="label">上游版本:</span>
            <span class="value" :class="{ 'text-success': entry.upstream_version }">
              {{ entry.upstream_version || '-' }}
            </span>
          </div>
          <div class="info-row">
            <span class="label">上次检查:</span>
            <span class="value text-muted">{{ fmtTimestamp(entry.upstream_last_checked) }}</span>
          </div>
        </div>

        <!-- 卡片操作 -->
        <div class="card-actions">
          <button
            class="action-btn action-view"
            @click.stop="emit('view', entry.pkgname)"
            title="查看详情"
          >
            <component :is="Icon.actionInfo" :size="14" />
          </button>
          <button
            class="action-btn action-edit"
            @click.stop="emit('edit', entry.pkgname)"
            title="编辑"
          >
            <component :is="Icon.actionEdit" :size="14" />
          </button>
          <button
            class="action-btn action-sync-aur"
            :disabled="isRowLoading?.(entry.pkgname, 'sync-aur')"
            @click.stop="emit('sync-aur', entry.pkgname)"
            title="同步AUR"
          >
            <component :is="Icon.syncAur" :size="14" />
          </button>
          <button
            class="action-btn action-check"
            :disabled="isRowLoading?.(entry.pkgname, 'check-upstream')"
            @click.stop="emit('check-upstream', entry.pkgname)"
            title="检查上游"
          >
            <component :is="Icon.actionSearch" :size="14" />
          </button>
          <button
            class="action-btn action-delete"
            @click.stop="emit('delete', entry.pkgname)"
            title="删除"
          >
            <component :is="Icon.actionDelete" :size="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- 分页控制 -->
    <div v-if="totalPages > 1" class="block-pagination">
      <button
        class="page-btn"
        :disabled="!currentPage || currentPage <= 1"
        @click="$emit('page-change', (currentPage || 1) - 1)"
      >
        上一页
      </button>
      <span class="page-info">
        第 {{ currentPage }} / {{ totalPages }} 页
      </span>
      <button
        class="page-btn"
        :disabled="!currentPage || currentPage >= totalPages"
        @click="$emit('page-change', (currentPage || 1) + 1)"
      >
        下一页
      </button>
    </div>
  </div>
</template>

<style scoped>
.block-view {
  width: 100%;
}

.block-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background-color: var(--bg-card);
  border-radius: 8px 8px 0 0;
  border: 1px solid var(--border);
  border-bottom: none;
}

.select-all-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
}

.block-count {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.block-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1rem;
  padding: 1rem;
  background-color: var(--bg-primary);
}

.block-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
  transition: all 0.2s;
  cursor: pointer;
}

.block-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.block-card.card-selected {
  border-color: var(--accent);
  background-color: var(--accent-light, rgba(59, 130, 246, 0.1));
}

.block-card.card-outdated .pkg-name {
  color: var(--warning);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.pkg-name {
  font-weight: 600;
  font-size: 1rem;
  color: var(--text-primary);
}

.status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
}

.badge-outdated {
  background-color: rgba(245, 158, 11, 0.15);
  color: var(--warning);
}

.badge-up-to-date {
  background-color: rgba(34, 197, 94, 0.15);
  color: var(--success);
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.8125rem;
}

.label {
  color: var(--text-secondary);
}

.value {
  color: var(--text-primary);
  font-family: monospace;
}

.text-success {
  color: var(--success);
}

.text-muted {
  color: var(--text-muted);
}

.card-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background-color: var(--bg-secondary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover:not(:disabled) {
  background-color: var(--accent);
  color: white;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.block-pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 1rem;
  background-color: var(--bg-card);
  border-radius: 0 0 8px 8px;
  border: 1px solid var(--border);
  border-top: none;
}

.page-btn {
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.15s;
}

.page-btn:hover:not(:disabled) {
  background-color: var(--accent);
  border-color: var(--accent);
  color: white;
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-info {
  font-size: 0.875rem;
  color: var(--text-secondary);
}
</style>
