<!--
  UpdateDiffDialog.vue - 软件更新的 dry-run 变更预览弹窗

  功能：按包展示 PKGBUILD 的 pkgver / pkgrel 变更 diff，确认后才允许写回。
  规约：内容区自然撑开、禁用滚动条；底部统一「确认 / 取消」两个图标+文本按钮；
       元信息用轻量背景块，不套带标题的卡片。
-->
<script setup lang="ts">
import { computed } from "vue";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";
import { Icon } from "../../icons";
import type { SoftwareUpdatePreview } from "../../types";

const props = defineProps<{
  /** 是否显示 */
  show: boolean;
  /** 是否正在生成预览 */
  loading: boolean;
  /** 预览条目（与选中的包名同序） */
  previews: SoftwareUpdatePreview[];
  /** 是否正在执行写回 */
  applying: boolean;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();

/** 可安全执行的条目（排除不可预览的错误项） */
const executable = computed(() => props.previews.filter((p) => !p.error));

/** 将纯文本 diff 拆成行，供逐行着色 */
function diffLines(text: string): string[] {
  return text.split("\n").filter((line) => line.length > 0);
}
</script>

<template>
  <StandardizedModal
    :show="show"
    width="lg"
    :scrollable="false"
    @close="emit('close')"
  >
    <template #header>
      <h3 class="modal-title">
        <component :is="Icon.updatePreview" :size="18" />
        变更预览（dry-run）
      </h3>
    </template>

    <div v-if="loading" class="preview-loading">正在生成预览…</div>
    <div v-else class="preview-list">
      <div
        v-for="p in previews"
        :key="p.pkgname"
        class="preview-item"
        :class="{ 'has-error': !!p.error }"
      >
        <div class="preview-head">
          <strong>{{ p.pkgname }}</strong>
          <span class="preview-path">{{ p.pkgbuild_path || "—" }}</span>
        </div>
        <p v-if="p.error" class="preview-error">
          <component :is="Icon.statusWarning" :size="14" />
          {{ p.error }}
        </p>
        <pre v-else class="preview-diff"><span
          v-for="(line, i) in diffLines(p.diff)"
          :key="i"
          :class="line.startsWith('+') ? 'diff-add' : 'diff-del'"
        >{{ line }}</span></pre>
      </div>
      <p v-if="previews.length === 0" class="preview-empty">没有可预览的条目</p>
    </div>

    <template #footer>
      <StandardizedButton variant="secondary" @click="emit('close')">
        <template #icon><component :is="Icon.actionClear" :size="16" /></template>
        取消
      </StandardizedButton>
      <StandardizedButton
        variant="primary"
        :disabled="executable.length === 0"
        :loading="applying"
        @click="emit('confirm')"
      >
        <template #icon><component :is="Icon.statusSuccess" :size="16" /></template>
        确认执行（{{ executable.length }}）
      </StandardizedButton>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.modal-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
  font-size: 1rem;
  color: var(--text-primary);
}

.preview-loading,
.preview-empty {
  padding: 1rem 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.preview-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.preview-item {
  padding: 0.6rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.preview-item.has-error {
  border-color: rgba(239, 83, 80, 0.5);
}

.preview-head {
  display: flex;
  align-items: baseline;
  gap: 0.6rem;
  margin-bottom: 0.4rem;
  color: var(--text-primary);
}

.preview-path {
  font-size: 0.75rem;
  color: var(--text-secondary);
  overflow-wrap: anywhere;
}

.preview-error {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  margin: 0;
  color: #e57373;
  font-size: 0.8125rem;
}

.preview-diff {
  margin: 0;
  padding: 0.5rem 0.6rem;
  background: var(--bg-primary);
  border-radius: 4px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 0.8125rem;
  white-space: pre-wrap;
}

.preview-diff span {
  display: block;
  padding: 0 0.25rem;
  color: var(--text-primary);
}

.diff-del {
  background: rgba(239, 83, 80, 0.12);
}

.diff-add {
  background: rgba(102, 187, 106, 0.12);
}
</style>
