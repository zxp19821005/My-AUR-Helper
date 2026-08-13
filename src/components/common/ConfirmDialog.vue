<!--
  ConfirmDialog.vue - 全局统一确认对话框

  功能：
  - 由 useConfirm.openConfirm 触发，Promise 化返回用户选择
  - 风格与项目其它弹窗一致（基于 StandardizedModal）
  - 底部两个带图标+文字的按钮：确认（statusSuccess 图标）/ 取消（actionClear 图标）
  - 点击关闭遮罩或“取消”均视为取消（resolve false）

  使用：在 App.vue 根节点挂载一次。
-->
<script setup lang="ts">
import { computed } from "vue";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";
import { Icon } from "../../icons";
import { confirmState, resolveConfirm } from "../../composables/useConfirm";

/** 确认按钮变体：danger 使用红色实心按钮，其余使用主色按钮 */
const confirmBtnVariant = computed(() =>
  confirmState.variant === "danger" ? "danger" : "primary"
);

/** 确认按钮语义色调：warning 用警告橙，其余不附加色调 */
const confirmBtnTone = computed(() =>
  confirmState.variant === "warning" ? "warning" : undefined
);

/** 用户点击确认 */
function onConfirm() {
  resolveConfirm(true);
}

/** 用户点击取消或关闭遮罩 */
function onCancel() {
  resolveConfirm(false);
}
</script>

<template>
  <Teleport to="body">
    <StandardizedModal
      :show="confirmState.visible"
      :title="confirmState.title"
      width="sm"
      @close="onCancel"
    >
      <div class="confirm-body">
        <p class="confirm-text">{{ confirmState.message }}</p>
      </div>
      <template #footer>
        <div class="confirm-footer">
          <StandardizedButton variant="secondary" @click="onCancel">
            <template #icon>
              <component :is="Icon.actionClear" :size="16" />
            </template>
            {{ confirmState.cancelText }}
          </StandardizedButton>
          <StandardizedButton :variant="confirmBtnVariant" :tone="confirmBtnTone" @click="onConfirm">
            <template #icon>
              <component :is="Icon.statusSuccess" :size="16" />
            </template>
            {{ confirmState.confirmText }}
          </StandardizedButton>
        </div>
      </template>
    </StandardizedModal>
  </Teleport>
</template>

<style scoped>
.confirm-body {
  padding: 4px 0;
}

.confirm-text {
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--text-primary);
  white-space: pre-line;
}

.confirm-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
