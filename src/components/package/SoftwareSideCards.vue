<!--
  SoftwareSideCards.vue - 软件侧边信息卡片组件

  功能：
  - 显示 AUR 信息（AUR版本、AUR License、最后提交）
  - 显示上游信息（上游版本、上游 License、上次检查）
-->
<script setup lang="ts">
import type { SoftwareDetail } from "../../types";
import { formatLicense, formatTimestamp } from "../../utils/format";

defineProps<{
  detail: SoftwareDetail;
}>();
</script>

<template>
  <div class="side-by-side">
    <div class="section half-section">
      <h4 class="section-title">AUR 信息</h4>
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">AUR 版本</td>
            <td class="value">{{ detail.aur_version || '—' }}</td>
          </tr>
          <tr>
            <td class="label">AUR License</td>
            <td class="value">
              {{ formatLicense(detail.aur_license_name) }}
            </td>
          </tr>
          <tr>
            <td class="label">最后提交</td>
            <td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="section half-section">
      <h4 class="section-title">上游信息</h4>
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">上游版本</td>
            <td class="value">{{ detail.upstream_version || '—' }}</td>
          </tr>
          <tr>
            <td class="label">上游 License</td>
            <td class="value">
              {{ formatLicense(detail.upstream_license_name) }}
            </td>
          </tr>
          <tr>
            <td class="label">上次检查</td>
            <td class="value">{{ formatTimestamp(detail.upstream_last_checked) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.side-by-side {
  display: flex;
  gap: 1rem;
}
.side-by-side .half-section {
  flex: 1;
  min-width: 0;
}

.section-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 0.5rem;
}

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