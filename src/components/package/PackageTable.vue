<!--
  PackageTable.vue - 软件包列表数据表格

  功能：
  - 基于 StandardizedTable 渲染软件包列表（含 AUR/上游版本与提交时间）
  - 提供行点击查看详情与行操作按钮组（查看/编辑/同步/更新/删除）
  - 通过 emit 将行操作转发给父视图的对应 action

  使用组件：
  - StandardizedTable: 通用数据表格
  - PackageRowActions: 行操作按钮组
-->
<script setup lang="ts">
import { fmtTimestamp } from "../../composables/usePackageList";
import type { Column } from "../../composables/useTableState";
import StandardizedTable from "../common/StandardizedTable.vue";
import PackageRowActions from "./PackageRowActions.vue";

const props = defineProps<{
  /** 过滤后的软件包条目 */
  entries: any[];
  /** 搜索关键字（表格内置搜索） */
  searchQuery: string;
  /** 每页条数 */
  pageSize: number;
  /** 当前页 */
  currentPage: number;
  /** 判断某行某操作是否正在执行 */
  isRowLoading: (pkgname: string, action: string) => boolean;
}>();

const emit = defineEmits<{
  (e: "row-click", pkgname: string): void;
  (e: "view", pkgname: string): void;
  (e: "edit", pkgname: string): void;
  (e: "sync-aur", pkgname: string): void;
  (e: "sync-pkgbuild", pkgname: string): void;
  (e: "check-upstream", pkgname: string): void;
  (e: "delete", pkgname: string): void;
}>();

/** 表格列配置 */
const columns: Column[] = [
  { key: "pkgname", title: "包名", sortable: true },
  { key: "aur_version", title: "AUR 版本", sortable: true },
  {
    key: "aur_last_updated",
    title: "AUR 最后提交",
    formatter: (value: any) => fmtTimestamp(value),
  },
  { key: "upstream_version", title: "上游版本", sortable: true },
  {
    key: "upstream_last_checked",
    title: "上游检查日期",
    formatter: (value: any) => fmtTimestamp(value),
  },
];
</script>

<template>
  <!-- key 必须固定：原 `:key="table-${entries.length}"` 会导致搜索/筛选/增删
       使条数变化时销毁重建整个表格。行级更新由 StandardizedTable 内部按
       rowKey="pkgname" 增量 diff，无需重建。 -->
  <StandardizedTable
    key="package-table"
    :columns="columns"
    :data="props.entries"
    :pageSize="props.pageSize"
    :searchQuery="props.searchQuery"
    :searchFields="['pkgname', 'aur_version', 'upstream_version']"
    :currentPage="props.currentPage"
    rowKey="pkgname"
    showCheckbox
    showIndex
    striped
    hoverable
    clickable
    :showPagination="false"
    emptyText="暂无软件包"
    @row-click="(row: any) => emit('row-click', row.pkgname)"
  >
    <template #cell-pkgname="{ row }">
      <strong :class="{ 'pkg-outdated': row.is_outdated }">
        {{ row.pkgname }}
      </strong>
    </template>

    <template #cell-aur_version="{ row }">
      {{ row.aur_version || "-" }}
    </template>

    <template #cell-aur_last_updated="{ row }">
      {{ fmtTimestamp(row.aur_last_updated) }}
    </template>

    <template #cell-upstream_version="{ row }">
      {{ row.upstream_version || "-" }}
    </template>

    <template #cell-upstream_last_checked="{ row }">
      {{ fmtTimestamp(row.upstream_last_checked) }}
    </template>

    <template #actions="{ row }">
      <PackageRowActions
        :pkgname="row.pkgname"
        :is-row-loading="props.isRowLoading"
        @view="(pkgname: string) => emit('view', pkgname)"
        @edit="(pkgname: string) => emit('edit', pkgname)"
        @sync-aur="(pkgname: string) => emit('sync-aur', pkgname)"
        @sync-pkgbuild="(pkgname: string) => emit('sync-pkgbuild', pkgname)"
        @check-upstream="(pkgname: string) => emit('check-upstream', pkgname)"
        @delete="(pkgname: string) => emit('delete', pkgname)"
      />
    </template>
  </StandardizedTable>
</template>

<style scoped>
.pkg-outdated {
  color: var(--warning);
}

/* 操作列按钮间距 */
:deep(.actions-cell) {
  gap: 0.25rem;
}
</style>
