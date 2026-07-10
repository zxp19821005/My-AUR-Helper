<!--
  DataTable.vue - 通用数据表格组件

  功能：
  - 支持列配置（字段名、标题、宽度、格式化函数）
  - 支持前端分页（通过 props 传入每页行数）
  - 支持搜索过滤
  - 支持行选择（单选/全选）
  - 支持自定义操作列
  - 支持自定义空状态
  - 即时响应设置变化，无需刷新

  使用示例：
  <DataTable
    :columns="columns"
    :data="allData"
    :pageSize="pageSize"
    :searchQuery="searchQuery"
    :searchFields="['spdx_id', 'full_name']"
    @selection-change="onSelectionChange"
  >
    <template #actions="{ row }">
      <button @click="edit(row)">编辑</button>
    </template>
  </DataTable>
-->
<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { Column } from "./DataTableTypes";
import DataTablePagination from "./DataTablePagination.vue";

export type { Column };

/** Props 接口 */
interface Props {
  /** 列配置 */
  columns: Column[];
  /** 数据源 */
  data: any[];
  /** 每页显示行数（0 表示不分页） */
  pageSize?: number;
  /** 搜索关键词 */
  searchQuery?: string;
  /** 搜索字段（用于指定搜索哪些字段） */
  searchFields?: string[];
  /** 行唯一标识字段（默认 'id'） */
  rowKey?: string;
  /** 是否显示复选框列 */
  showCheckbox?: boolean;
  /** 是否显示序号列 */
  showIndex?: boolean;
  /** 空状态提示文本 */
  emptyText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  pageSize: 50,
  searchQuery: "",
  searchFields: () => [],
  rowKey: "id",
  showCheckbox: false,
  showIndex: false,
  emptyText: "暂无数据",
});

/** Events */
const emit = defineEmits<{
  /** 选择变化事件 */
  (e: "selection-change", selectedRows: any[]): void;
  /** 行点击事件 */
  (e: "row-click", row: any): void;
}>();

/** 当前页码 */
const currentPage = ref(1);

/** 选中的行 */
const selectedRows = ref(new Set<any>());

/** 搜索过滤后的数据 */
const filteredData = computed(() => {
  if (!props.searchQuery || props.searchFields.length === 0) {
    return props.data;
  }
  const query = props.searchQuery.toLowerCase();
  return props.data.filter((row) =>
    props.searchFields.some((field) => {
      const value = row[field];
      if (value == null) return false;
      return String(value).toLowerCase().includes(query);
    })
  );
});

/** 总记录数 */
const totalRecords = computed(() => filteredData.value.length);

/** 总页数 */
const totalPages = computed(() => {
  if (props.pageSize <= 0) return 1;
  return Math.ceil(totalRecords.value / props.pageSize);
});

/** 当前页数据 */
const pageData = computed(() => {
  if (props.pageSize <= 0) return filteredData.value;
  const start = (currentPage.value - 1) * props.pageSize;
  return filteredData.value.slice(start, start + props.pageSize);
});

/** 是否全选当前页 */
const isAllSelected = computed(() => {
  if (pageData.value.length === 0) return false;
  return pageData.value.every((row) => selectedRows.value.has(row[props.rowKey]));
});

/** 是否部分选中（用于 indeterminate 状态） */
const isPartialSelected = computed(() => {
  if (pageData.value.length === 0) return false;
  const selectedCount = pageData.value.filter((row) =>
    selectedRows.value.has(row[props.rowKey])
  ).length;
  return selectedCount > 0 && selectedCount < pageData.value.length;
});

/** 格式化单元格内容 */
function formatCell(value: any, column: Column, row: any): string {
  if (column.formatter) {
    return column.formatter(value, row);
  }
  if (value == null || value === "") return "-";
  return String(value);
}

/** 切换单行选择 */
function toggleSelect(rowKey: any) {
  const newSet = new Set(selectedRows.value);
  if (newSet.has(rowKey)) {
    newSet.delete(rowKey);
  } else {
    newSet.add(rowKey);
  }
  selectedRows.value = newSet;
  emitSelectionChange();
}

/** 切换全选/取消全选 */
function toggleSelectAll() {
  if (isAllSelected.value) {
    const newSet = new Set(selectedRows.value);
    pageData.value.forEach((row) => newSet.delete(row[props.rowKey]));
    selectedRows.value = newSet;
  } else {
    const newSet = new Set(selectedRows.value);
    pageData.value.forEach((row) => newSet.add(row[props.rowKey]));
    selectedRows.value = newSet;
  }
  emitSelectionChange();
}

/** 跳转到指定页 */
function goToPage(page: number) {
  if (page < 1 || page > totalPages.value) return;
  currentPage.value = page;
}

/** 触发选择变化事件 */
function emitSelectionChange() {
  const selected = props.data.filter((row) =>
    selectedRows.value.has(row[props.rowKey])
  );
  emit("selection-change", selected);
}

/** 处理行点击 */
function handleRowClick(row: any) {
  emit("row-click", row);
}

/** 清空选择 */
function clearSelection() {
  selectedRows.value = new Set();
  emitSelectionChange();
}

/** 搜索关键词变化时重置页码 */
watch(
  () => props.searchQuery,
  () => { currentPage.value = 1; }
);

/** 每页行数变化时重置页码 */
watch(
  () => props.pageSize,
  () => { currentPage.value = 1; }
);

/** 暴露方法给父组件 */
defineExpose({ clearSelection, goToPage, selectedRows });
</script>

<template>
  <div class="data-table-wrapper">
    <div class="card" style="overflow-x: auto">
      <table class="data-table">
        <thead>
          <tr>
            <th v-if="showCheckbox" style="width: 2.5rem">
              <input type="checkbox" :checked="isAllSelected" :indeterminate="isPartialSelected" @change="toggleSelectAll" />
            </th>
            <th v-if="showIndex" style="width: 3rem">#</th>
            <th v-for="col in columns" :key="col.key" :style="{ width: col.width || 'auto', textAlign: col.align || 'left' }">
              {{ col.title }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, index) in pageData" :key="row[rowKey]" @click="handleRowClick(row)">
            <td v-if="showCheckbox" @click.stop>
              <input type="checkbox" :checked="selectedRows.has(row[rowKey])" @change="toggleSelect(row[rowKey])" />
            </td>
            <td v-if="showIndex" class="index-cell">{{ (currentPage - 1) * pageSize + index + 1 }}</td>
            <td v-for="col in columns" :key="col.key" :style="{ textAlign: col.align || 'left' }">
              <slot :name="`cell-${col.key}`" :row="row" :value="row[col.key]">
                {{ formatCell(row[col.key], col, row) }}
              </slot>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="pageData.length === 0" class="empty-state"><p>{{ emptyText }}</p></div>
    </div>
    <DataTablePagination :currentPage="currentPage" :totalRecords="totalRecords" :totalPages="totalPages" @goToPage="goToPage" />
  </div>
</template>

<style scoped>
.data-table-wrapper { display: flex; flex-direction: column; gap: 0.5rem; }
.data-table { width: 100%; border-collapse: collapse; table-layout: auto; }
.data-table th { text-align: left; padding: 0.75rem; color: var(--text-secondary); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; border-bottom: 1px solid var(--border); white-space: nowrap; }
.data-table td { padding: 0.75rem; border-bottom: 1px solid var(--border); font-size: 0.875rem; }
.data-table tbody tr { cursor: pointer; transition: background-color 0.15s; }
.data-table tbody tr:hover { background-color: rgba(108, 99, 255, 0.05); }
.index-cell { color: var(--text-secondary); font-size: 0.75rem; }
.empty-state { padding: 2rem; text-align: center; color: var(--text-secondary); }
</style>
