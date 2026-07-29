<script setup lang="ts">
import { computed } from "vue";
import type { SoftwareDetail } from "../../types";
import StandardizedCard from "../base/StandardizedCard.vue";

defineProps<{
  upstreamVersion: string | null;
  upstreamLastChecked: number | null;
  detail?: SoftwareDetail;
}>();

function fmtDate(ts: number | null): string {
  if (ts == null) return "—";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
}
</script>

<template>
  <StandardizedCard
    title="上游版本信息"
    subtitle="最新版本检查结果"
    layout="table"
  >
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">上游版本</td>
          <td class="value version-cell">{{ upstreamVersion || '—' }}</td>
        </tr>
        <tr>
          <td class="label">上次检查</td>
          <td class="value">{{ fmtDate(upstreamLastChecked) }}</td>
        </tr>
      </tbody>
    </table>
  </StandardizedCard>
</template>
