<!--
  StandardizedModal.vue - 增强版通用模态框组件

  功能：
  - 统一模态框视觉风格
  - 支持多种尺寸（sm/md/lg/xl/full）
  - 支持自定义标题、内容、底部操作区
  - 支持错误/警告/成功状态提示
  - 支持点击遮罩关闭
  - 支持ESC键关闭
  - 支持拖拽移动（可选）
  - 支持动画过渡效果
  - 支持全屏模式
  - 支持滚动锁定

  Props:
  - show: boolean - 是否显示
  - title?: string - 模态框标题
  - width?: 'sm' | 'md' | 'lg' | 'xl' | 'full' | string - 模态框宽度
  - hideHeader?: boolean - 是否隐藏头部
  - hideFooter?: boolean - 是否隐藏底部
  - closable?: boolean - 是否允许关闭（点击遮罩/ESC）
  - closeOnEsc?: boolean - 是否允许ESC键关闭
  - closeOnOverlay?: boolean - 是否允许点击遮罩关闭
  - scrollable?: boolean - 内容是否可滚动
  - draggable?: boolean - 是否可拖拽

  Events:
  - close - 关闭模态框
  - update:show - 显示状态变化

  Slots:
  - default - 模态框主体内容
  - header - 自定义头部
  - footer - 自定义底部操作区
  - close - 自定义关闭按钮
  - error - 错误提示区域

  使用示例：
  <StandardizedModal
    v-model:show="showModal"
    title="编辑软件包"
    width="lg"
    @close="handleClose"
  >
    <template #footer>
      <button class="btn btn-secondary" @click="showModal = false">取消</button>
      <button class="btn btn-primary" @click="handleSave">保存</button>
    </template>
    <form>...</form>
  </StandardizedModal>
-->
<script setup lang="ts">
import { X } from "@lucide/vue";
import { watch, onMounted, onUnmounted } from "vue";

const props = withDefaults(defineProps<{
  /** 是否显示 */
  show?: boolean;
  /** 模态框标题 */
  title?: string;
  /** 模态框宽度 */
  width?: "sm" | "md" | "lg" | "xl" | "full" | string;
  /** 是否隐藏头部 */
  hideHeader?: boolean;
  /** 是否隐藏底部 */
  hideFooter?: boolean;
  /** 是否允许关闭 */
  closable?: boolean;
  /** 是否允许ESC键关闭 */
  closeOnEsc?: boolean;
  /** 是否允许点击遮罩关闭 */
  closeOnOverlay?: boolean;
  /** 内容是否可滚动 */
  scrollable?: boolean;
  /** 是否可拖拽 */
  draggable?: boolean;
}>(), {
  show: false,
  title: "",
  width: "md",
  hideHeader: false,
  hideFooter: false,
  closable: true,
  closeOnEsc: true,
  closeOnOverlay: true,
  scrollable: true,
  draggable: false,
});

const emit = defineEmits<{
  close: [];
  "update:show": [value: boolean];
}>();


/** 关闭模态框 */
function handleClose() {
  if (props.closable) {
    emit("close");
    emit("update:show", false);
  }
}

/** 处理ESC键 */
function handleEscKey(event: KeyboardEvent) {
  if (event.key === "Escape" && props.show && props.closeOnEsc) {
    handleClose();
  }
}

/** 滚动锁定 */
watch(
  () => props.show,
  (val) => {
    if (val) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
  }
);

/** 注册ESC键监听 */
onMounted(() => {
  window.addEventListener("keydown", handleEscKey);
});

/** 移除ESC键监听 */
onUnmounted(() => {
  window.removeEventListener("keydown", handleEscKey);
  document.body.style.overflow = "";
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-overlay" @click.self="closeOnOverlay ? handleClose() : null">
        <div
          ref="modalRef"
          class="modal"
          :class="[
            `modal-${width}`,
            { 'modal-draggable': draggable },
          ]"
        >
          <!-- 头部 -->
          <div v-if="!hideHeader" class="modal-header">
            <div class="modal-header-left">
              <slot name="header-left" />
              <h3 v-if="title" class="modal-title">{{ title }}</h3>
              <slot name="header" />
            </div>
            <button
              v-if="closable"
              class="modal-close"
              type="button"
              @click="handleClose"
              title="关闭"
            >
              <slot name="close">
                <X :size="18" />
              </slot>
            </button>
          </div>

          <!-- 错误提示 -->
          <div v-if="$slots.error" class="modal-error">
            <slot name="error" />
          </div>

          <!-- 主体内容 -->
          <div class="modal-body" :class="{ 'modal-body-scrollable': scrollable }">
            <slot />
          </div>

          <!-- 底部操作区 -->
          <div v-if="!hideFooter && $slots.footer" class="modal-footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
  padding: 1rem;
}

.modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  width: 100%;
  max-height: 85vh;
  transition: all 0.3s ease;
}

/* 宽度变体 */
.modal-sm {
  max-width: 400px;
  min-width: 320px;
}

.modal-md {
  max-width: 500px;
  min-width: 400px;
}

.modal-lg {
  max-width: 700px;
  min-width: 500px;
}

.modal-xl {
  max-width: 900px;
  min-width: 600px;
}

.modal-full {
  max-width: 95vw;
  width: 95vw;
  height: 90vh;
  max-height: 90vh;
}

/* 可拖拽 */
.modal-draggable {
  cursor: move;
}

/* 头部 */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  min-width: 0;
}

.modal-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 6px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.modal-close:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

/* 错误提示 */
.modal-error {
  padding: 0.75rem 1.25rem;
  color: var(--error);
  font-size: 0.875rem;
  background: var(--error-bg);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

/* 主体内容 */
.modal-body {
  padding: 1.25rem;
  flex: 1;
  position: relative;
  overflow: hidden;
}

.modal-body-scrollable {
  overflow-y: auto;
}

/* 底部操作区 */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem 1.25rem;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

.modal-enter-to,
.modal-leave-from {
  opacity: 1;
  transform: scale(1) translateY(0);
}

.modal-overlay-enter-active,
.modal-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.modal-overlay-enter-from,
.modal-overlay-leave-to {
  opacity: 0;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-overlay {
    padding: 0.5rem;
    align-items: flex-end;
  }

  .modal {
    max-height: 90vh;
    border-radius: 12px 12px 0 0;
  }

  .modal-sm,
  .modal-md,
  .modal-lg,
  .modal-xl {
    max-width: 100%;
    min-width: 100%;
  }

  .modal-header {
    padding: 0.75rem 1rem;
  }

  .modal-body {
    padding: 1rem;
  }

  .modal-footer {
    padding: 0.75rem 1rem;
    flex-direction: column;
  }

  .modal-footer > * {
    width: 100%;
  }
}
</style>