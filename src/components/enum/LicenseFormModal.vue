<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  show: boolean;
  mode: "add" | "edit";
  license: {
    id: number | null;
    spdx_id: string;
    full_name: string;
  };
}>();

const emit = defineEmits<{
  save: [data: { id: number | null; spdx_id: string; full_name: string }];
  close: [];
}>();

const form = ref({ ...props.license });

watch(() => props.show, (val) => {
  if (val) form.value = { ...props.license };
});
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-card">
      <h3 style="margin: 0 0 1rem 0">{{ mode === 'add' ? '新增 License' : '编辑 License' }}</h3>
      <div class="form-group">
        <label>SPDX ID</label>
        <input v-model="form.spdx_id" class="form-input" placeholder="如 MIT, GPL-3.0-only" />
      </div>
      <div class="form-group">
        <label>完整名称</label>
        <input v-model="form.full_name" class="form-input" placeholder="如 MIT License" />
      </div>
      <div style="display: flex; gap: 0.5rem; justify-content: flex-end; margin-top: 1rem">
        <button class="btn btn-outline" @click="emit('close')">取消</button>
        <button class="btn btn-primary" @click="emit('save', form)" :disabled="!form.spdx_id.trim() || !form.full_name.trim()">确认</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal-card {
  background: var(--bg-primary);
  border-radius: 12px;
  padding: 1.5rem;
  min-width: 400px;
  max-width: 500px;
}
.form-group { margin-bottom: 0.75rem; }
.form-group label {
  display: block;
  font-size: 0.8125rem;
  margin-bottom: 0.25rem;
  color: var(--text-secondary);
}
.form-input {
  width: 100%;
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  box-sizing: border-box;
}
</style>
