<!--
  SoftwareSideCards.vue - 软件侧边信息卡片组件

  功能：
  - 显示 AUR 信息和上游版本信息（左右并排）
-->
<script setup lang="ts">
import type { SoftwareDetail } from "../types";
import SoftwareAurCard from "./SoftwareAurCard.vue";
import SoftwareUpstreamCard from "./SoftwareUpstreamCard.vue";
import { formatLicense } from "../utils/format";

defineProps<{
  detail: SoftwareDetail;
}>();
</script>

<template>
  <div class="side-by-side">
    <div class="section half-section">
      <h4 class="section-title">AUR 信息（扩展）</h4>
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">AUR License</td>
            <td class="value">
              {{ formatLicense(detail.aur_license_name) }}
            </td>
          </tr>
        </tbody>
      </table>
      <SoftwareAurCard
        :aur-version="detail.aur_version"
        :aur-pkgdesc="detail.aur_pkgdesc"
        :aur-last-updated="detail.aur_last_updated"
      />
    </div>

    <div class="section half-section">
      <h4 class="section-title">上游版本信息（扩展）</h4>
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">上游 License</td>
            <td class="value">
              {{ formatLicense(detail.upstream_license_name) }}
            </td>
          </tr>
        </tbody>
      </table>
      <SoftwareUpstreamCard
        :upstream-version="detail.upstream_version"
        :upstream-last-checked="detail.upstream_last_checked"
      />
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