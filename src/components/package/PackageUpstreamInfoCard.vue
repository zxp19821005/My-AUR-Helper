<!--
  PackageUpstreamInfoCard.vue - 软件包上游信息卡片

  功能：
  - 显示上游版本、上游检查时间
  - 显示上游 URL
-->
<script setup lang="ts">
import { Info, Clock } from "@lucide/vue";
import type { SoftwareDetail } from "../../types";
import StandardizedCard from "../base/StandardizedCard.vue";

defineProps<{
  detail: SoftwareDetail;
}>();

function fmtTimestamp(ts: number | null): string {
  if (!ts) return "-";
  const d = new Date(ts * 1000);
  return d.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <StandardizedCard
    title="上游信息"
    :subtitle="detail.upstream_version || '未知'"
    layout="table"
  >
    <template #status>
      <Info :size="16" />
    </template>
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">上游版本</td>
          <td class="value version-value">{{ detail.upstream_version || "-" }}</td>
        </tr>
        <tr>
          <td class="label">上游检查日期</td>
          <td class="value">
            <Clock :size="12" class="inline-icon" />
            {{ fmtTimestamp(detail.upstream_last_checked) }}
          </td>
        </tr>
        <tr>
          <td class="label">上游 URL</td>
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

.version-value {
  font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
  font-weight: 500;
}

.url-value a {
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}

.url-value a:hover {
  text-decoration: underline;
}

.inline-icon {
  display: inline-block;
  vertical-align: middle;
  margin-right: 0.25rem;
  opacity: 0.6;
}
</style>