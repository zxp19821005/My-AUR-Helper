<script setup lang="ts">
import type { SoftwareDetail } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
import { computed } from "vue";

const props = defineProps<{
  detail: SoftwareDetail;
}>();

const pkgTypeName = computed(() => {
  return pkgTypeOptions.find(t => t.id === props.detail.package_type_id)?.label || '未知';
});

const checkerTypeName = computed(() => {
  return checkerTypeOptions.find(c => c.id === props.detail.checker_type_id)?.label || '未知';
});
</script>

<template>
  <div class="info-card">
    <div class="card-header">
      <h3 class="card-title">基本信息</h3>
      <span :class="['status-badge', detail.is_outdated ? 'outdated' : 'latest']">
        {{ detail.is_outdated ? '需更新' : '已最新' }}
      </span>
    </div>
    <div class="badge-row">
      <span class="type-tag">{{ pkgTypeName }}</span>
      <span class="type-tag">{{ checkerTypeName }}</span>
    </div>
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">包名</td>
          <td class="value">{{ detail.pkgname }}</td>
        </tr>
        <tr>
          <td class="label">上游地址</td>
          <td class="value url-value">
            <a v-if="detail.upstream_url" :href="detail.upstream_url" target="_blank">
              {{ detail.upstream_url }}
            </a>
            <span v-else>未设置</span>
          </td>
        </tr>
        <tr>
          <td class="label">包描述</td>
          <td class="value">{{ detail.aur_pkgdesc || '—' }}</td>
        </tr>
        <tr>
          <td class="label">版本提取关键字</td>
          <td class="value">
            <code v-if="detail.version_extract_regex">{{ detail.version_extract_regex }}</code>
            <span v-else class="empty">未设置</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
