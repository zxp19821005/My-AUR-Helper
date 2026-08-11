<!--
  ProxyEditModal.vue - 代理编辑弹窗

  功能：修改代理名称，保存时 emit save(name) 由父组件调用更新接口。
  使用组件：StandardizedModal
-->
<script setup lang="ts">
import { ref, watch } from "vue";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";
import type { ProxyInfo } from "../../types";

const props = defineProps<{
  show: boolean;
  proxy: ProxyInfo | null;
}>();
const emit = defineEmits<{
  close: [];
  save: [name: string];
}>();

const editName = ref("");

// 打开弹窗时同步当前代理名称
watch(
  () => props.proxy,
  (p) => { if (p) editName.value = p.proxy_name; },
  { immediate: true }
);

function handleSave() {
  emit("save", editName.value.trim());
}
</script>

<template>
  <StandardizedModal :show="show" title="编辑代理" width="sm" @close="emit('close')">
    <div class="modal-form">
      <div class="form-group">
        <label class="form-label">代理名称</label>
        <input
          v-model="editName"
          type="text"
          class="form-input"
          placeholder="输入代理名称"
        />
      </div>
    </div>
    <template #footer>
      <div class="modal-footer-actions">
        <StandardizedButton variant="secondary" @click="emit('close')">取消</StandardizedButton>
        <StandardizedButton variant="primary" @click="handleSave">保存</StandardizedButton>
      </div>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.modal-form { padding: 4px 0; }
.form-group { display: flex; flex-direction: column; gap: 6px; }
.form-label { font-size: 13px; font-weight: 500; color: var(--text-primary); }
.form-input {
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
}
.form-input:focus { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(108,99,255,.15); }
.modal-footer-actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
