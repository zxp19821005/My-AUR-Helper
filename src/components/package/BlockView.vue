<!--
  BlockView.vue - 软件包块视图组件

  功能：
  - 以卡片形式展示软件包列表
  - 支持选中、查看详情、编辑、同步、检查、删除等操作
  - 响应式网格布局

  参考：Clash-verge 连接页面风格
  样式已外置至 BlockView.css
-->
<script setup lang="ts">
import { computed } from "vue";
import type { SoftwareListEntry } from "@/types";
import { fmtTimestamp } from "@/utils/format";
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

<style scoped src="./BlockView.css"></style>
