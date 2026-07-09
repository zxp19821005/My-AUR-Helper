<script setup lang="ts">
defineProps<{
  upstreamVersion: string | null;
  upstreamLastChecked: number | null;
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
  <div class="info-card">
    <div class="card-header">
      <h3 class="card-title">上游版本信息</h3>
      <span class="card-subtitle">最新版本检查结果</span>
    </div>
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
  </div>
</template>
