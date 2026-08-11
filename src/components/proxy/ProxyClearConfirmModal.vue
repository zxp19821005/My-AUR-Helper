<!--
  ProxyClearConfirmModal.vue - 清空代理表确认弹窗

  功能：二次确认清空所有代理数据，确认时 emit confirm() 由父组件执行。
  使用组件：StandardizedModal
-->
<script setup lang="ts">
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

defineProps<{
  show: boolean;
  clearing: boolean;
}>();
const emit = defineEmits<{
  close: [];
  confirm: [];
}>();
</script>

<template>
  <StandardizedModal :show="show" title="确认清空" width="sm" @close="emit('close')">
    <div class="modal-form">
      <p class="confirm-text">确定要清空所有代理数据吗？此操作不可恢复。</p>
      <p class="confirm-sub">将清空 <code>proxies_info</code> 和 <code>proxies_test</code> 表，并重置 proxy_id。</p>
    </div>
    <template #footer>
      <div class="modal-footer-actions">
        <StandardizedButton variant="secondary" @click="emit('close')">取消</StandardizedButton>
        <StandardizedButton variant="danger" :disabled="clearing" @click="emit('confirm')">确定清空</StandardizedButton>
      </div>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.modal-form { padding: 4px 0; }
.confirm-text { font-size: 14px; color: var(--text-primary); margin-bottom: 8px; }
.confirm-sub { font-size: 12px; color: var(--text-muted); }
.confirm-sub code { background: var(--bg-secondary); padding: 1px 4px; border-radius: 3px; font-size: 11px; }
.modal-footer-actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
