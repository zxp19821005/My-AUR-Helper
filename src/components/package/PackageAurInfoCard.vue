<!--
  PackageAurInfoCard.vue - 软件包 AUR 信息卡片

  功能：
  - 显示 AUR 版本、最后提交时间
  - 显示依赖、构建依赖、可选依赖
-->
<script setup lang="ts">
import { Globe, Clock } from "@lucide/vue";
import type { SoftwareDetail } from "../../types";
import StandardizedCard from "../components/base/StandardizedCard.vue";

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
    title="AUR 信息"
    :subtitle="detail.aur_version || '未知'"
    layout="table"
  >
    <template #status>
      <Globe :size="16" />
    </template>
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">AUR 版本</td>
          <td class="value version-value">{{ detail.aur_version || "-" }}</td>
        </tr>
        <tr>
          <td class="label">AUR 最后提交</td>
          <td class="value">
            <Clock :size="12" class="inline-icon" />
            {{ fmtTimestamp(detail.aur_last_updated) }}
          </td>
        </tr>
        <tr>
          <td class="label">依赖</td>
          <td class="value">{{ detail.depends || "-" }}</td>
        </tr>
        <tr>
          <td class="label">构建依赖</td>
          <td class="value">{{ detail.makedepends || "-" }}</td>
        </tr>
        <tr>
          <td class="label">可选依赖</td>
          <td class="value">{{ detail.optdepends || "-" }}</td>
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

.inline-icon {
  display: inline-block;
  vertical-align: middle;
  margin-right: 0.25rem;
  opacity: 0.6;
}
</style>