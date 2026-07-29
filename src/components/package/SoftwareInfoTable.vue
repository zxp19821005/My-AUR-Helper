<!--
  SoftwareInfoTable.vue - 软件信息表格组件

  功能：
  - 显示运行时依赖、构建依赖、可选依赖、编程语言
-->
<script setup lang="ts">
import type { SoftwareDetail, Language } from "../../types";
import { parseJsonList, getLanguageNames } from "../../utils/format";

defineProps<{
  detail: SoftwareDetail;
  languages: Language[];
}>();
</script>

<template>
  <table class="info-table">
    <tbody>
      <tr>
        <td class="label">运行时依赖</td>
        <td class="value">{{ parseJsonList(detail.depends) }}</td>
      </tr>
      <tr>
        <td class="label">构建依赖</td>
        <td class="value">{{ parseJsonList(detail.makedepends) }}</td>
      </tr>
      <tr>
        <td class="label">可选依赖</td>
        <td class="value">{{ parseJsonList(detail.optdepends) }}</td>
      </tr>
      <tr>
        <td class="label">编程语言</td>
        <td class="value">
          {{ getLanguageNames(detail.language_ids, languages) }}
        </td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.info-table {
  width: 100%;
  border-collapse: collapse;
}
.info-table .label {
  width: 120px;
  padding: 0.5rem 0;
  font-size: 0.8125rem;
  color: var(--text-secondary);
  vertical-align: top;
}
.info-table .value {
  padding: 0.5rem 0;
  font-size: 0.875rem;
  color: var(--text-primary);
}
</style>