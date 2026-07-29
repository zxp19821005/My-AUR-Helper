<!--
  SoftwareStatusRow.vue - 软件状态行组件

  功能：
  - 显示自动检查、测试版本、二进制文件的状态
-->
<script setup lang="ts">
import type { SoftwareDetail } from "../types";

defineProps<{
  detail: SoftwareDetail;
}>();
</script>

<template>
  <div class="status-row">
    <span class="status-item">
      <span class="status-label">自动检查</span>
      <span
        :class="[
          'status-value',
          detail.auto_check_enabled ? 'enabled' : 'disabled',
        ]"
      >
        {{ detail.auto_check_enabled ? "已启用" : "已禁用" }}
      </span>
    </span>
    <span class="status-item">
      <span class="status-label">测试版本</span>
      <span
        :class="[
          'status-value',
          detail.check_test_versions ? 'enabled' : 'disabled',
        ]"
      >
        {{ detail.check_test_versions ? "已启用" : "已禁用" }}
      </span>
    </span>
    <span class="status-item">
      <span class="status-label">二进制文件</span>
      <span
        :class="[
          'status-value',
          detail.check_binary_files ? 'enabled' : 'disabled',
        ]"
      >
        {{ detail.check_binary_files ? "已启用" : "已禁用" }}
      </span>
    </span>
  </div>
</template>

<style scoped>
.status-row {
  display: flex;
  gap: 1.25rem;
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.status-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.status-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.status-value {
  font-size: 0.75rem;
  font-weight: 500;
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
}

.status-value.enabled {
  color: var(--success);
  background: var(--success-bg);
}

.status-value.disabled {
  color: var(--error);
  background: var(--error-bg);
}
</style>