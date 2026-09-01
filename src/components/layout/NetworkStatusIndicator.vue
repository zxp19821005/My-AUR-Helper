/**
 * NetworkStatusIndicator.vue - 网络状态指示器
 *
 * 固定在页面右上角，实时显示：
 * - 浏览器网络状态（在线/离线）
 * - AUR 服务可达性（可达/不可达/检测中）
 *
 * 状态颜色：
 * - 绿色：在线 + AUR 可达
 * - 黄色：在线但 AUR 不可达
 * - 红色：浏览器离线
 * - 灰色：检测中
 *
 * 点击可手动触发一次 AUR 连通性检测
 */
<script setup lang="ts">
import { useNetworkStatus } from "../../composables/useNetworkStatus";
import { Icon } from "../../icons";

const { isOnline, isAurOnline, checkAurConnectivity } = useNetworkStatus();

/** 获取状态描述文本 */
function getStatusText(): string {
  if (!isOnline.value) return "网络已断开";
  if (isAurOnline.value === null) return "正在检测 AUR...";
  if (isAurOnline.value) return "网络正常，AUR 可达";
  return "网络正常，但 AUR 暂不可达";
}

/** 获取状态颜色 class */
function getStatusClass(): string {
  if (!isOnline.value) return "status-offline";
  if (isAurOnline.value === null) return "status-checking";
  return isAurOnline.value ? "status-online" : "status-warning";
}

/** 获取图标令牌 */
function getStatusIcon() {
  if (!isOnline.value) return Icon.statusError;
  if (isAurOnline.value === null) return Icon.loading;
  return isAurOnline.value ? Icon.statusSuccess : Icon.statusWarning;
}

let checking = false;
async function handleRefresh() {
  if (checking || !isOnline.value) return;
  checking = true;
  isAurOnline.value = null;
  await checkAurConnectivity();
  checking = false;
}
</script>

<template>
  <div
    class="network-indicator"
    :class="[getStatusClass(), { disabled: !isOnline }]"
    :title="getStatusText()"
    @click="handleRefresh"
  >
    <component :is="getStatusIcon()" :size="14" />
    <span class="indicator-label">{{ getStatusText() }}</span>
  </div>
</template>

<style scoped>
.network-indicator {
  position: fixed;
  top: 0.75rem;
  right: 0.75rem;
  z-index: 9999;
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.65rem;
  border-radius: 6px;
  font-size: 0.75rem;
  color: white;
  cursor: default;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.18);
  transition: opacity 0.3s, transform 0.2s;
  user-select: none;
}

.network-indicator.disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.network-indicator.status-online {
  background: var(--success, #22c55e);
}

.network-indicator.status-warning {
  background: var(--warning, #f59e0b);
}

.network-indicator.status-offline {
  background: var(--danger, #ef4444);
}

.network-indicator.status-checking {
  background: var(--muted, #6b7280);
  animation: pulse 1.5s ease-in-out infinite;
}

.indicator-label {
  white-space: nowrap;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.55;
  }
}
</style>
