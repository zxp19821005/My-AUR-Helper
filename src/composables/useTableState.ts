/**
 * useTableState.ts - 表格状态管理组合式函数
 *
 * 功能：
 * - 管理表格分页、排序、选择状态
 * - 提供搜索过滤、排序逻辑
 * - 提供选择/全选切换方法
 * - 提供分页导航方法
 *
 * 使用场景：
 * - StandardizedTable 组件内部使用
 * - 需要表格状态管理的自定义表格
 */
import { computed, ref, watch } from "vue";
import { ChevronUp, ChevronDown, ChevronsUpDown } from "@lucide/vue";

export interface Column {
  key: string;
  title: string;
  width?: string;
  formatter?: (value: any, row: any) => string;
  align?: "left" | "center" | "right";
  sortable?: boolean;
  hidden?: boolean;
}

interface UseTableStateProps {
  columns: Column[];
  data: any[] | (() => any[]);
  pageSize: number | (() => number);
  searchQuery: string | (() => string);
  searchFields: string[] | (() => string[]);
  rowKey: string;
  currentPage?: number | (() => number | undefined);
  totalRecords?: number;
}

export function useTableState(props: UseTableStateProps) {
  const currentPage = ref(1);
  const selectedRows = ref(new Set<any>());
  const sortState = ref<{ key: string; direction: "asc" | "desc" | null }>({
    key: "",
    direction: null,
  });

  const resolvedData = computed(() => {
    return typeof props.data === "function" ? props.data() : props.data;
  });

  const resolvedSearchQuery = computed(() => {
    return typeof props.searchQuery === "function" ? props.searchQuery() : props.searchQuery;
  });

  const resolvedSearchFields = computed(() => {
    return typeof props.searchFields === "function" ? props.searchFields() : props.searchFields;
  });

  const resolvedPageSize = computed(() => {
    return typeof props.pageSize === "function" ? props.pageSize() : props.pageSize;
  });

  watch(
    resolvedData,
    () => {
      currentPage.value = 1;
    },
    { deep: true }
  );

  const visibleColumns = computed(() =>
    props.columns.filter((col) => !col.hidden)
  );

  const filteredData = computed(() => {
    let result = resolvedData.value;

    if (resolvedSearchQuery.value && resolvedSearchFields.value.length > 0) {
      const query = resolvedSearchQuery.value.toLowerCase();
      result = result.filter((row) =>
        resolvedSearchFields.value.some((field) => {
          const value = row[field];
          if (value == null) return false;
          return String(value).toLowerCase().includes(query);
        })
      );
    }

    if (sortState.value.key && sortState.value.direction) {
      const { key, direction } = sortState.value;
      result = [...result].sort((a, b) => {
        const valA = a[key];
        const valB = b[key];
        if (valA == null && valB == null) return 0;
        if (valA == null) return direction === "asc" ? 1 : -1;
        if (valB == null) return direction === "asc" ? -1 : 1;

        if (typeof valA === "number" && typeof valB === "number") {
          return direction === "asc" ? valA - valB : valB - valA;
        }

        const strA = String(valA).toLowerCase();
        const strB = String(valB).toLowerCase();
        return direction === "asc"
          ? strA.localeCompare(strB)
          : strB.localeCompare(strA);
      });
    }

    return result;
  });

  const totalRecords = computed(() => filteredData.value.length);

  const totalPages = computed(() => {
    if (resolvedPageSize.value <= 0) return 1;
    return Math.ceil(totalRecords.value / resolvedPageSize.value);
  });

  const pageData = computed(() => {
    if (resolvedPageSize.value <= 0) return filteredData.value;
    const start = (currentPage.value - 1) * resolvedPageSize.value;
    return filteredData.value.slice(start, start + resolvedPageSize.value);
  });

  const isAllSelected = computed(() => {
    if (pageData.value.length === 0) return false;
    return pageData.value.every((row) =>
      selectedRows.value.has(row[props.rowKey])
    );
  });

  const isPartialSelected = computed(() => {
    if (pageData.value.length === 0) return false;
    const selectedCount = pageData.value.filter((row) =>
      selectedRows.value.has(row[props.rowKey])
    ).length;
    return selectedCount > 0 && selectedCount < pageData.value.length;
  });

  function formatCell(value: any, column: Column, row: any): string {
    if (column.formatter) {
      return column.formatter(value, row);
    }
    if (value == null || value === "") return "-";
    return String(value);
  }

  function toggleSelect(rowKey: any, emitSelectionChange: () => void) {
    const newSet = new Set(selectedRows.value);
    if (newSet.has(rowKey)) {
      newSet.delete(rowKey);
    } else {
      newSet.add(rowKey);
    }
    selectedRows.value = newSet;
    emitSelectionChange();
  }

  function toggleSelectAll(emitSelectionChange: () => void) {
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

  function goToPage(page: number, emitPageChange: (page: number) => void) {
    if (page < 1 || page > totalPages.value) return;
    currentPage.value = page;
    emitPageChange(page);
  }

  function handleSort(
    column: Column,
    emitSortChange: (key: string, direction: "asc" | "desc" | null) => void
  ) {
    if (!column.sortable) return;

    if (sortState.value.key === column.key) {
      if (sortState.value.direction === "asc") {
        sortState.value.direction = "desc";
      } else if (sortState.value.direction === "desc") {
        sortState.value.direction = null;
        sortState.value.key = "";
      }
    } else {
      sortState.value.key = column.key;
      sortState.value.direction = "asc";
    }
    emitSortChange(sortState.value.key, sortState.value.direction);
  }

  function getSortIcon(column: Column) {
    if (sortState.value.key !== column.key) return ChevronsUpDown;
    if (sortState.value.direction === "asc") return ChevronUp;
    if (sortState.value.direction === "desc") return ChevronDown;
    return ChevronsUpDown;
  }

  function getPageNumbers(): (number | string)[] {
    const total = totalPages.value;
    const current = currentPage.value;
    const pages: (number | string)[] = [];

    if (total <= 7) {
      for (let i = 1; i <= total; i++) pages.push(i);
    } else {
      pages.push(1, 2, 3);
      if (current > 5) pages.push("...");
      for (
        let i = Math.max(4, current - 1);
        i <= Math.min(total - 2, current + 1);
        i++
      ) {
        pages.push(i);
      }
      if (current < total - 4) pages.push("...");
      pages.push(total - 2, total - 1, total);
    }

    return pages;
  }

  function clearSelection(emitSelectionChange: () => void) {
    selectedRows.value = new Set();
    emitSelectionChange();
  }

  function resetSort(
    emitSortChange: (key: string, direction: "asc" | "desc" | null) => void
  ) {
    sortState.value = { key: "", direction: null };
    emitSortChange("", null);
  }

  watch(resolvedSearchQuery, () => {
    currentPage.value = 1;
  });

  watch(resolvedPageSize, () => {
    currentPage.value = 1;
  });

  watch(
    () => {
      const cp = props.currentPage;
      return typeof cp === "function" ? cp() : cp;
    },
    (newPage) => {
      if (newPage != null && newPage !== currentPage.value) {
        currentPage.value = newPage;
      }
    }
  );

  return {
    currentPage,
    selectedRows,
    sortState,
    visibleColumns,
    filteredData,
    totalRecords,
    totalPages,
    pageData,
    isAllSelected,
    isPartialSelected,
    formatCell,
    toggleSelect,
    toggleSelectAll,
    goToPage,
    handleSort,
    getSortIcon,
    getPageNumbers,
    clearSelection,
    resetSort,
  };
}