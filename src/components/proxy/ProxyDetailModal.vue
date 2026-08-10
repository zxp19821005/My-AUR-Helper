<!--
  ProxyDetailModal.vue - 代理详情弹窗

  功能：展示单个代理的完整信息（名称、URL、类型、状态、统计）。
  使用组件：StandardizedModal、StandardizedBadge
-->
<script setup lang="ts">
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedBadge from "../base/StandardizedBadge.vue";
import { getProxyDisplayName } from "../../composables/useProxyList";
import type { ProxyInfo } from "../../types";

defineProps<{
  show: boolean;
  proxy: ProxyInfo | null;
}>();
const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <StandardizedModal :show="show" title="代理详情" width="md" @close="emit('close')">
    <div v-if="proxy" class="detail-content">
      <div class="detail-row">
        <span class="detail-label">名称</span>
        <span class="detail-value">{{ getProxyDisplayName(proxy) }}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">URL</span>
        <span class="detail-value url-value">{{ proxy.url }}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">代理类型</span>
        <StandardizedBadge
          :text="proxy.proxy_type"
          :class="`type-${proxy.proxy_type}`"
          size="sm"
          variant="soft"
        />
      </div>
      <div class="detail-row">
        <span class="detail-label">状态</span>
        <StandardizedBadge
          :type="proxy.is_active ? 'success' : 'neutral'"
          :text="proxy.is_active ? '启用' : '禁用'"
          size="sm"
        />
      </div>
      <div class="detail-row">
        <span class="detail-label">成功次数</span>
        <span class="detail-value">{{ proxy.success_count ?? 0 }}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">失败次数</span>
        <span class="detail-value">{{ proxy.fail_count ?? 0 }}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">平均延迟</span>
        <span class="detail-value">{{ proxy.avg_latency !== null ? `${proxy.avg_latency}ms` : '未测试' }}</span>
      </div>
    </div>
  </StandardizedModal>
</template>

<style scoped>
.detail-content { display: flex; flex-direction: column; gap: 12px; padding: 4px 0; }
.detail-row { display: flex; align-items: center; gap: 12px; }
.detail-label {
  min-width: 80px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted, #6b7280);
  flex-shrink: 0;
}
.detail-value { font-size: 13px; color: var(--text-primary, #374151); }
.url-value { word-break: break-all; }
</style>
