<!--
  StandardizedTable.vue - 增强版通用数据表格组件

  功能：
  - 支持列配置（字段名、标题、宽度、对齐、格式化函数、自定义渲染）
  - 支持前端分页（可配置每页行数）
  - 支持搜索过滤（可配置搜索字段）
  - 支持行选择（单选/全选/部分选中）
  - 支持自定义操作列（通过插槽）
  - 支持自定义空状态
  - 支持加载状态
  - 支持行点击事件
  - 支持条纹样式
  - 支持紧凑模式
  - 即时响应设置变化，无需刷新

  Props:
  - columns: Column[] - 列配置
  - data: any[] - 数据源
  - pageSize?: number - 每页显示行数（0表示不分页）
  - searchQuery?: string - 搜索关键词
  - searchFields?: string[] - 搜索字段
  - rowKey?: string - 行唯一标识字段（默认'id'）
  - showCheckbox?: boolean - 是否显示复选框列
  - showIndex?: boolean - 是否显示序号列
  - emptyText?: string - 空状态提示文本
  - loading?: boolean - 是否显示加载状态
  - striped?: boolean - 是否显示条纹
  - compact?: boolean - 是否紧凑模式
  - hoverable?: boolean - 是否启用悬停效果
  - clickable?: boolean - 是否启用行点击

  Events:
  - selection-change - 选择变化事件
  - row-click - 行点击事件
  - page-change - 页码变化事件
  - sort-change - 排序变化事件

  Slots:
  - cell-{key} - 自定义单元格内容
  - actions - 操作列内容
  - empty - 自定义空状态
-->
<script setup lang="ts">
import { computed } from "vue";
import { useTableState, type Column } from "../../composables/useTableState";
import StandardizedTableHeader from "./StandardizedTableHeader.vue";
import StandardizedTableRow from "./StandardizedTableRow.vue";
import StandardizedTablePagination from "./StandardizedTablePagination.vue";

interface Props {
  columns: Column[];
  data: any[];
  pageSize?: number;
  searchQuery?: string;
  searchFields?: string[];
  rowKey?: string;
  showCheckbox?: boolean;
  showIndex?: boolean;
  emptyText?: string;
  loading?: boolean;
  striped?: boolean;
  compact?: boolean;
  hoverable?: boolean;
  clickable?: boolean;
  showPagination?: boolean;
  currentPage?: number;
}

const props = withDefaults(defineProps<Props>(), {
  pageSize: 50,
  searchQuery: "",
  searchFields: () => [],
  rowKey: "id",
  showCheckbox: false,
  showIndex: false,
  emptyText: "暂无数据",
  loading: false,
  striped: false,
  compact: false,
  hoverable: true,
  clickable: false,
  showPagination: true,
});

const emit = defineEmits<{
  (e: "selection-change", selectedRows: any[]): void;
  (e: "row-click", row: any): void;
  (e: "page-change", page: number): void;
  (e: "sort-change", key: string, direction: "asc" | "desc" | null): void;
}>();

function emitSelectionChange() {
  const selected = props.data.filter((row) =>
    tableState.selectedRows.value.has(row[props.rowKey])
  );
  emit("selection-change", selected);
}

const tableState = useTableState({
  columns: props.columns,
  data: () => props.data,
  pageSize: props.pageSize,
  searchQuery: props.searchQuery,
  searchFields: props.searchFields,
  rowKey: props.rowKey,
  currentPage: () => props.currentPage,
});

const currentPage = computed(() => tableState.currentPage.value);
const selectedRows = computed(() => tableState.selectedRows.value);
const visibleColumns = computed(() => tableState.visibleColumns.value);
const pageData = computed(() => tableState.pageData.value);
const totalRecords = computed(() => tableState.totalRecords.value);
const totalPages = computed(() => tableState.totalPages.value);
const isAllSelected = computed(() => tableState.isAllSelected.value);
const isPartialSelected = computed(() => tableState.isPartialSelected.value);
const sortState = computed(() => tableState.sortState.value);

function handleToggleSelect(rowKey: any) {
  tableState.toggleSelect(rowKey, emitSelectionChange);
}

function handleToggleSelectAll() {
  tableState.toggleSelectAll(emitSelectionChange);
}

function handleGoToPage(page: number) {
  tableState.goToPage(page, (p) => emit("page-change", p));
}

function handleSort(column: Column) {
  tableState.handleSort(column, (k, d) => emit("sort-change", k, d));
}

function handleRowClick(row: any) {
  if (props.clickable) {
    emit("row-click", row);
  }
}

function clearSelection() {
  tableState.clearSelection(emitSelectionChange);
}

function resetSort() {
  tableState.resetSort((k, d) => emit("sort-change", k, d));
}

defineExpose({
  clearSelection,
  goToPage: handleGoToPage,
  resetSort,
  selectedRows,
  pageData,
  totalRecords,
  totalPages,
});
</script>

<template>
  <div class="table-wrapper" :class="{ 'table-loading': loading }">
    <div v-if="loading" class="table-loading-overlay">
      <div class="table-loading-spinner">加载中...</div>
    </div>

    <div class="table-container" :class="{ 'table-compact': compact }">
      <table
        class="standardized-table"
        :class="{ 'table-striped': striped, 'table-hoverable': hoverable }"
      >
        <StandardizedTableHeader
          :visible-columns="visibleColumns"
          :show-checkbox="showCheckbox"
          :show-index="showIndex"
          :is-all-selected="isAllSelected"
          :is-partial-selected="isPartialSelected"
          :sort-state="sortState"
          :get-sort-icon="tableState.getSortIcon"
          :has-actions="!!$slots.actions"
          @toggle-select-all="handleToggleSelectAll"
          @sort="handleSort"
        />
        <tbody>
          <StandardizedTableRow
            v-for="(row, index) in pageData"
            :key="row[props.rowKey]"
            :row="row"
            :index="index"
            :row-key="props.rowKey"
            :visible-columns="visibleColumns"
            :selected-rows="selectedRows"
            :show-checkbox="showCheckbox"
            :show-index="showIndex"
            :current-page="currentPage"
            :page-size="props.pageSize"
            :striped="striped"
            :clickable="clickable"
            :format-cell="tableState.formatCell"
            :has-actions="!!$slots.actions"
            @toggle-select="handleToggleSelect"
            @row-click="handleRowClick"
          >
            <template
              v-for="col in visibleColumns"
              :key="col.key"
              #[`cell-${col.key}`]="slotProps"
            >
              <slot
                :name="`cell-${col.key}`"
                :row="slotProps.row"
                :value="slotProps.value"
                :index="slotProps.index"
              >
                {{ slotProps.value }}
              </slot>
            </template>
            <template #actions="slotProps">
              <slot name="actions" :row="slotProps.row" :index="slotProps.index" />
            </template>
          </StandardizedTableRow>
        </tbody>
      </table>

      <div
        v-if="pageData.length === 0 && !loading"
        class="table-empty"
      >
        <slot name="empty">
          <p>{{ emptyText }}</p>
        </slot>
      </div>
    </div>

    <StandardizedTablePagination
      v-if="props.showPagination && totalRecords > 0 && props.pageSize > 0"
      :current-page="currentPage"
      :total-pages="totalPages"
      :total-records="totalRecords"
      @page-change="handleGoToPage"
    />
  </div>
</template>

<style scoped>
.table-wrapper {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.table-container {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--bg-card);
}

.table-compact .standardized-table th,
.table-compact .standardized-table td {
  padding: 0.5rem 0.75rem;
}

.table-loading-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  border-radius: 8px;
}

.table-loading-spinner {
  padding: 1rem 2rem;
  background: var(--bg-card);
  border-radius: 8px;
  color: var(--text-primary);
  font-weight: 500;
}
</style>