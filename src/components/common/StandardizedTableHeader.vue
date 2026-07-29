<!--
  StandardizedTableHeader.vue - 表格表头组件

  功能：
  - 渲染表头列
  - 支持复选框全选
  - 支持序号列
  - 支持排序列
  - 支持操作列
-->
<script setup lang="ts">
import type { Column } from "../../composables/useTableState";

defineProps<{
  visibleColumns: Column[];
  showCheckbox: boolean;
  showIndex: boolean;
  isAllSelected: boolean;
  isPartialSelected: boolean;
  sortState: { key: string; direction: "asc" | "desc" | null };
  getSortIcon: (col: Column) => any;
  hasActions: boolean;
}>();

const emit = defineEmits<{
  "toggle-select-all": [];
  sort: [column: Column];
}>();

function handleSort(col: Column) {
  emit("sort", col);
}
</script>

<template>
  <thead>
    <tr>
      <th v-if="showCheckbox" class="checkbox-col">
        <input
          type="checkbox"
          :checked="isAllSelected"
          :indeterminate="isPartialSelected"
          @change="emit('toggle-select-all')"
        />
      </th>
      <th v-if="showIndex" class="index-col">#</th>
      <th
        v-for="col in visibleColumns"
        :key="col.key"
        :class="{ sortable: col.sortable }"
        :style="{ width: col.width || 'auto', textAlign: col.align || 'left' }"
        @click="handleSort(col)"
      >
        <span class="th-content">
          {{ col.title }}
          <component
            v-if="col.sortable"
            :is="getSortIcon(col)"
            :size="12"
            class="sort-icon"
            :class="{
              'sort-active':
                sortState.key === col.key &&
                sortState.direction,
            }"
          />
        </span>
      </th>
      <th v-if="hasActions" class="actions-col">操作</th>
    </tr>
  </thead>
</template>

<style scoped>
.checkbox-col,
.index-col,
.actions-col {
  width: 3rem;
  text-align: center;
}

.sortable {
  cursor: pointer;
  user-select: none;
}

.sortable:hover {
  color: var(--text-primary);
}

.th-content {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.sort-icon {
  color: var(--text-muted);
  transition: color 0.15s;
}

.sort-active {
  color: var(--accent);
}
</style>