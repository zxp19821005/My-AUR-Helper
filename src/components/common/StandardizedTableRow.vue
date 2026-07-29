<!--
  StandardizedTableRow.vue - 表格行组件

  功能：
  - 渲染单行数据
  - 支持复选框选择
  - 支持序号显示
  - 支持自定义单元格内容（插槽）
  - 支持行点击事件
  - 支持选中/条纹样式
-->
<script setup lang="ts">
import type { Column } from "../../composables/useTableState";

const props = defineProps<{
  row: any;
  index: number;
  rowKey: string;
  visibleColumns: Column[];
  selectedRows: Set<any>;
  showCheckbox: boolean;
  showIndex: boolean;
  currentPage: number;
  pageSize: number;
  striped: boolean;
  clickable: boolean;
  formatCell: (value: any, column: Column, row: any) => string;
  hasActions: boolean;
}>();

const emit = defineEmits<{
  "toggle-select": [rowKey: any];
  "row-click": [row: any];
}>();

function handleRowClick() {
  if (props.clickable) {
    emit("row-click", props.row);
  }
}

function isSelected(): boolean {
  return props.selectedRows.has(props.row[props.rowKey]);
}
</script>

<template>
  <tr
    :class="{
      'row-selected': isSelected(),
      'row-striped': striped && index % 2 === 1,
    }"
    @click="handleRowClick"
  >
    <td v-if="showCheckbox" class="checkbox-col" @click.stop>
      <input
        type="checkbox"
        :checked="isSelected()"
        @change="emit('toggle-select', row[rowKey])"
      />
    </td>
    <td v-if="showIndex" class="index-cell">
      {{ (currentPage - 1) * pageSize + index + 1 }}
    </td>
    <td
      v-for="col in visibleColumns"
      :key="col.key"
      :style="{ textAlign: col.align || 'left' }"
    >
      <slot
        :name="`cell-${col.key}`"
        :row="row"
        :value="row[col.key]"
        :index="index"
      >
        {{ formatCell(row[col.key], col, row) }}
      </slot>
    </td>
    <td v-if="hasActions" class="actions-cell" @click.stop>
      <slot name="actions" :row="row" :index="index" />
    </td>
  </tr>
</template>

<style scoped>
.checkbox-col,
.index-col,
.actions-col {
  width: 3rem;
  text-align: center;
}

.index-cell {
  color: var(--text-secondary);
  font-size: 0.75rem;
  text-align: center;
}

.actions-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
}

.row-selected {
  background-color: rgba(108, 99, 255, 0.1) !important;
}
</style>