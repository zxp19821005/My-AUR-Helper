<script setup lang="ts">
defineProps<{
  aurVersion: string | null;
  aurPkgdesc: string | null;
  aurLastUpdated: number | null;
}>();

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString("zh-CN");
}
</script>

<template>
  <div class="info-card">
    <div class="card-header">
      <h3 class="card-title">AUR 信息</h3>
      <span class="card-subtitle">来自 AUR 仓库</span>
    </div>
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
  </div>
</template>
