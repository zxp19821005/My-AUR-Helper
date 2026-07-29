<!--
  PackageBasicInfoCard.vue - 软件包基本信息卡片

  功能：
  - 显示软件包基本信息（ID、包名、描述、许可证、上游地址、版本提取正则）
  - 显示包类型和检查器类型标签
-->
<script setup lang="ts">
import { computed } from "vue";
import { Package } from "@lucide/vue";
import type { SoftwareDetail } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
import StandardizedCard from "../components/base/StandardizedCard.vue";

const props = defineProps<{
  detail: SoftwareDetail;
}>();

const pkgTypeName = computed(() => {
  return pkgTypeOptions.find(t => t.id === props.detail.package_type_id)?.label || "未知";
});

const checkerTypeName = computed(() => {
  return checkerTypeOptions.find(c => c.id === props.detail.checker_type_id)?.label || "未知";
});
</script>

<template>
  <StandardizedCard
    title="基本信息"
    :subtitle="detail.pkgname"
    layout="table"
  >
    <template #status>
      <Package :size="16" />
    </template>
    <div class="badge-row">
      <span class="type-tag">{{ pkgTypeName }}</span>
      <span class="type-tag">{{ checkerTypeName }}</span>
    </div>
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">软件包ID</td>
          <td class="value">{{ detail.software_id || "-" }}</td>
        </tr>
        <tr>
          <td class="label">包名</td>
          <td class="value">{{ detail.pkgname }}</td>
        </tr>
        <tr>
          <td class="label">AUR 描述</td>
          <td class="value">{{ detail.aur_pkgdesc || "-" }}</td>
        </tr>
        <tr>
          <td class="label">许可证</td>
          <td class="value">{{ detail.aur_license_name || "-" }}</td>
        </tr>
        <tr>
          <td class="label">上游地址</td>
          <td class="value url-value">
            <a
              v-if="detail.upstream_url"
              :href="detail.upstream_url"
              target="_blank"
              rel="noopener noreferrer"
            >
              {{ detail.upstream_url }}
            </a>
            <span v-else>-</span>
          </td>
        </tr>
        <tr>
          <td class="label">版本提取正则</td>
          <td class="value">
            <code v-if="detail.version_extract_regex" class="code-value">{{ detail.version_extract_regex }}</code>
            <span v-else class="empty-value">未设置</span>
          </td>
        </tr>
      </tbody>
    </table>
  </StandardizedCard>
</template>

<style scoped>
.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 0.5rem 0.25rem;
  font-size: 0.8125rem;
}

.info-table .label {
  width: 100px;
  color: var(--text-secondary);
  vertical-align: top;
  font-weight: 500;
}

.info-table .value {
  color: var(--text-primary);
}

.badge-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.type-tag {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.code-value {
  font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
  font-size: 0.75rem;
  background: var(--bg-secondary);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  color: var(--accent);
}

.empty-value {
  color: var(--text-muted, var(--text-secondary));
  font-style: italic;
}

.url-value a {
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}

.url-value a:hover {
  text-decoration: underline;
}
</style>