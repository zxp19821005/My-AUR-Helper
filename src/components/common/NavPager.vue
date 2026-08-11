<!--
  NavPager.vue - 通用上/下一个导航组件

  功能：
  - 统一"上一个 / 下一个"导航按钮的视觉与交互（accent 紫底 + 白图标，常驻颜色）
  - 支持两种布局：
    - floating：绝对定位覆盖在相对父级左右、垂直居中（详情弹窗悬浮导航）
    - inline：流式排列（用于页面 header，可带文字标签）

  使用场景：软件管理详情弹窗、备份管理详情弹窗、软件管理详情页 header
-->
<script setup lang="ts">
import { Icon } from "../../icons";

withDefaults(defineProps<{
  /** 上一个目标（null 表示不可用，按钮禁用） */
  prev?: string | null;
  /** 下一个目标（null 表示不可用，按钮禁用） */
  next?: string | null;
  /** 布局：floating=悬浮覆盖 / inline=行内 */
  variant?: "floating" | "inline";
  /** 是否显示文字标签（inline 模式常用） */
  showLabels?: boolean;
}>(), {
  prev: null,
  next: null,
  variant: "floating",
  showLabels: false,
});

const emit = defineEmits<{
  /** 导航方向 */
  navigate: [direction: "prev" | "next"];
}>();
</script>

<template>
  <div class="nav-pager" :class="`nav-pager--${variant}`">
    <button
      class="nav-btn"
      :class="{ disabled: !prev }"
      :disabled="!prev"
      @click="emit('navigate', 'prev')"
      title="上一个"
    >
      <component :is="Icon.arrowLeft" :size="20" />
      <span v-if="showLabels && prev" class="nav-label">{{ prev }}</span>
    </button>
    <button
      class="nav-btn"
      :class="{ disabled: !next }"
      :disabled="!next"
      @click="emit('navigate', 'next')"
      title="下一个"
    >
      <span v-if="showLabels && next" class="nav-label">{{ next }}</span>
      <component :is="Icon.arrowRight" :size="20" />
    </button>
  </div>
</template>
