<script setup lang="ts">
defineProps<{
  upstreamVersion: string | null;
  upstreamLastChecked: string | null;
}>();
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
          <td class="value">{{ upstreamLastChecked || '—' }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
