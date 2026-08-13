<!--
  CacheSudoersModal.vue - 缓存清理 sudoers 免密配置提示弹窗

  功能：缓存清理需要 root 权限，提示用户配置 sudoers 免密规则，
  并提供一键复制命令按钮。

  说明：基于 StandardizedModal 统一模态框风格（取代原先自写的 modal-overlay 结构）
-->
<script setup lang="ts">
import { inject } from "vue";
import { FOOTER_KEY, addMessage } from "../../composables/footer";
import { Icon } from "../../icons";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

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
  <StandardizedModal
    :show="show"
    title="需要配置 sudoers 免密权限"
    width="sm"
    @close="emit('close')"
  >
    <div class="sudoers-body">
      <p>缓存清理功能需要 root 权限来清理系统缓存 /var/cache/pacman/pkg。</p>
      <p>请在终端中执行以下命令来配置免密权限：</p>
      <pre class="sudoers-command">{{ sudoersCommand }}</pre>
      <p class="hint">配置完成后，请重新启动应用。</p>
    </div>

    <template #footer>
      <StandardizedButton variant="outline" size="sm" @click="emit('close')">
        <template #icon>
          <component :is="Icon.actionClear" :size="16" />
        </template>
        取消
      </StandardizedButton>
      <StandardizedButton variant="primary" size="sm" @click="copySudoersCommand">
        <template #icon>
          <component :is="Icon.actionCopy" :size="16" />
        </template>
        复制命令
      </StandardizedButton>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.sudoers-body p {
  margin: 0 0 0.75rem 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}
.sudoers-body p:last-of-type {
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
</style>
