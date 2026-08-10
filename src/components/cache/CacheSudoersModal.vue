<!--
  CacheSudoersModal.vue - 缓存清理 sudoers 免密配置提示弹窗

  功能：缓存清理需要 root 权限，提示用户配置 sudoers 免密规则，
  并提供一键复制命令按钮。
  使用组件：Teleport（挂载到 body，避免被父级 overflow 裁剪）
-->
<script setup lang="ts">
import { inject } from "vue";
import { X } from "@lucide/vue";
import { FOOTER_KEY, addMessage } from "../../composables/footer";

const props = defineProps<{
  show: boolean;
  sudoersCommand: string;
}>();
const emit = defineEmits<{
  close: [];
}>();

const footer = inject(FOOTER_KEY)!;

/** 复制 sudoers 命令到剪贴板 */
async function copySudoersCommand() {
  try {
    await navigator.clipboard.writeText(props.sudoersCommand);
    addMessage(footer, "success", "sudoers 配置命令已复制到剪贴板");
  } catch {
    addMessage(footer, "error", "复制命令失败，请手动复制");
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content">
        <div class="modal-header">
          <h3>需要配置 sudoers 免密权限</h3>
          <button class="btn-icon btn-icon-default" @click="emit('close')">
            <X :size="16" />
          </button>
        </div>
        <div class="modal-body">
          <p>缓存清理功能需要 root 权限来清理系统缓存 /var/cache/pacman/pkg。</p>
          <p>请在终端中执行以下命令来配置免密权限：</p>
          <pre class="sudoers-command">{{ sudoersCommand }}</pre>
          <p class="hint">配置完成后，请重新启动应用。</p>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="emit('close')">取消</button>
          <button class="btn btn-primary" @click="copySudoersCommand">复制命令</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background-color: var(--bg-primary);
  border-radius: 12px;
  border: 1px solid var(--border);
  width: 90%;
  max-width: 500px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}

.modal-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-body {
  padding: 1.25rem;
}

.modal-body p {
  margin: 0 0 0.75rem 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}

.modal-body p:last-of-type {
  margin-bottom: 0;
}

.sudoers-command {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.75rem;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 0.8rem;
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0.75rem 0;
}

.hint {
  color: var(--text-muted);
  font-style: italic;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-top: 1px solid var(--border);
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.btn-secondary {
  background-color: var(--bg-secondary);
  border-color: var(--border);
  color: var(--text-primary);
}

.btn-secondary:hover {
  background-color: var(--bg-tertiary);
}

.btn-primary {
  background-color: var(--accent);
  color: white;
}

.btn-primary:hover {
  opacity: 0.9;
}
</style>
