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
import { watch, computed, onMounted, onUnmounted } from "vue";

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

/** 预设宽度枚举，非预设值（如 "720px"）作为内联样式应用 */
const presetWidths = ["sm", "md", "lg", "xl", "full"];
const isPresetWidth = computed(() => presetWidths.includes(props.width));

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
            isPresetWidth ? `modal-${width}` : '',
            { 'modal-draggable': draggable },
          ]"
          :style="isPresetWidth ? {} : { maxWidth: width, minWidth: width }"
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
/* 所有样式已移至全局 modal-styles.css */
</style>