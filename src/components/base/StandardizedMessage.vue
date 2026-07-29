<!--
  StandardizedMessage.vue - 通用消息提示组件

  功能：
  - 统一消息提示样式（成功、错误、警告、信息）
  - 支持自动消失
  - 支持手动关闭
  - 支持自定义图标
  - 支持全局消息队列

  Props:
  - type?: 'success' | 'error' | 'warning' | 'info' - 消息类型
  - message?: string - 消息内容
  - duration?: number - 自动消失时间（毫秒），0表示不自动消失
  - closable?: boolean - 是否显示关闭按钮
  - show?: boolean - 是否显示

  Events:
  - close - 关闭消息
  - update:show - 显示状态变化

  使用示例：
  <StandardizedMessage
    type="success"
    message="保存成功"
    :duration="3000"
    closable
  />
-->
<script setup lang="ts">
import { CheckCircle, XCircle, AlertTriangle, Info, X } from "@lucide/vue";
import { ref, watch, onMounted } from "vue";

const props = withDefaults(defineProps<{
  /** 消息类型 */
  type?: "success" | "error" | "warning" | "info";
  /** 消息内容 */
  message?: string;
  /** 自动消失时间（毫秒），0表示不自动消失 */
  duration?: number;
  /** 是否显示关闭按钮 */
  closable?: boolean;
  /** 是否显示 */
  show?: boolean;
}>(), {
  type: "info",
  message: "",
  duration: 3000,
  closable: true,
  show: true,
});

const emit = defineEmits<{
  close: [];
  "update:show": [value: boolean];
}>();

const visible = ref(props.show);
let timer: ReturnType<typeof setTimeout> | null = null;

/** 消息类型配置 */
const typeConfig = {
  success: {
    icon: CheckCircle,
    color: "var(--success)",
    bgColor: "var(--success-bg)",
    borderColor: "var(--success)",
  },
  error: {
    icon: XCircle,
    color: "var(--error)",
    bgColor: "var(--error-bg)",
    borderColor: "var(--error)",
  },
  warning: {
    icon: AlertTriangle,
    color: "var(--warning)",
    bgColor: "var(--warning-bg)",
    borderColor: "var(--warning)",
  },
  info: {
    icon: Info,
    color: "var(--accent)",
    bgColor: "rgba(108, 99, 255, 0.15)",
    borderColor: "var(--accent)",
  },
};

/** 关闭消息 */
function handleClose() {
  visible.value = false;
  if (timer) clearTimeout(timer);
  emit("close");
  emit("update:show", false);
}

/** 自动消失逻辑 */
watch(
  () => props.show,
  (val) => {
    visible.value = val;
    if (val && props.duration > 0) {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        handleClose();
      }, props.duration);
    }
  }
);

/** 组件挂载时启动定时器 */
onMounted(() => {
  if (visible.value && props.duration > 0) {
    timer = setTimeout(() => {
      handleClose();
    }, props.duration);
  }
});
</script>

<template>
  <Transition name="message">
    <div
      v-if="visible"
      class="message"
      :class="`message-${type}`"
      :style="{
        '--msg-color': typeConfig[type].color,
        '--msg-bg': typeConfig[type].bgColor,
        '--msg-border': typeConfig[type].borderColor,
      }"
    >
      <component
        :is="typeConfig[type].icon"
        :size="18"
        class="message-icon"
        :style="{ color: typeConfig[type].color }"
      />
      <span class="message-text">{{ message }}</span>
      <button
        v-if="closable"
        class="message-close"
        type="button"
        @click="handleClose"
        title="关闭"
      >
        <X :size="14" />
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.message {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--msg-border);
  background-color: var(--msg-bg);
  color: var(--msg-color);
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.2s;
}

.message-icon {
  flex-shrink: 0;
}

.message-text {
  flex: 1;
  color: var(--text-primary);
}

.message-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.message-close:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}

/* 消息过渡动画 */
.message-enter-active,
.message-leave-active {
  transition: all 0.3s ease;
}

.message-enter-from,
.message-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.message-enter-to,
.message-leave-from {
  opacity: 1;
  transform: translateY(0);
}
</style>