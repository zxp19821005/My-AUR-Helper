<script setup lang="ts">
import { computed } from "vue";
import type { SoftwareDetail } from "../../types";
import StandardizedCard from "./base/StandardizedCard.vue";
import StandardizedBadge from "./base/StandardizedBadge.vue";

defineProps<{
  aurVersion: string | null;
  aurPkgdesc: string | null;
  aurLastUpdated: number | null;
  detail?: SoftwareDetail;
}>();

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
}

const statusOptions = computed(() => [
  { value: "latest", text: "已最新", className: "status-badge-success" },
  { value: "outdated", text: "需更新", className: "status-badge-warning" },
]);
</script>

<template>
  <StandardizedCard
    title="AUR 信息"
    subtitle="来自 AUR 仓库"
    :status="detail?.is_outdated ? 'outdated' : 'latest'"
    :statusOptions="statusOptions"
    layout="table"
  >
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">AUR 版本</td>
          <td class="value version-cell">{{ aurVersion || '—' }}</td>
        </tr>
        <tr>
          <td class="label">包描述</td>
          <td class="value">{{ aurPkgdesc || '—' }}</td>
        </tr>
        <tr>
          <td class="label">更新时间</td>
          <td class="value">{{ formatTimestamp(aurLastUpdated) }}</td>
        </tr>
      </tbody>
    </table>
  </StandardizedCard>
</template>
