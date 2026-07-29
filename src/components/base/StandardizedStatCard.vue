<!--
  StandardizedStatCard.vue - 通用统计卡片组件

  功能：
  - 显示统计数据（数值、标题、图标）
  - 支持趋势指示器（上升/下降）
  - 支持点击交互
  - 统一视觉风格

  Props:
  - title: string - 卡片标题
  - value: string | number - 统计数值
  - icon?: Component - 图标组件（可选）
  - trend?: 'up' | 'down' | 'neutral' - 趋势方向（可选）
  - trendValue?: string - 趋势数值（可选）
  - color?: string - 主题色（使用CSS变量名）
  - clickable?: boolean - 是否可点击

  Slots:
  - default: 自定义内容区域
  - footer: 底部内容区域

  使用示例：
  <StandardizedStatCard
    title="总包数"
    :value="1234"
    :icon="Package"
    color="var(--accent)"
    clickable
    @click="navigateToPackages"
  />
-->
<script setup lang="ts">
import { ArrowUp, ArrowDown, Minus } from "@lucide/vue";
import { computed, type Component } from "vue";

const props = withDefaults(defineProps<{
  /** 卡片标题 */
  title: string;
  /** 统计数值 */
  value: string | number;
  /** 图标组件 */
  icon?: Component;
  /** 趋势方向 */
  trend?: "up" | "down" | "neutral";
  /** 趋势数值 */
  trendValue?: string;
  /** 主题色（CSS变量名） */
  color?: string;
  /** 是否可点击 */
  clickable?: boolean;
}>(), {
  trend: "neutral",
  clickable: false,
});

const emit = defineEmits<{
  click: [];
}>();

function handleClick() {
  if (props.clickable) {
    emit("click");
  }
}

const trendIcon = computed(() => {
  switch (props.trend) {
    case "up": return ArrowUp;
    case "down": return ArrowDown;
    default: return Minus;
  }
});

const trendColor = computed(() => {
  switch (props.trend) {
    case "up": return "var(--success)";
    case "down": return "var(--error)";
    default: return "var(--text-muted)";
  }
});
</script>

<template>
  <div
    class="stat-card"
    :class="{ 'stat-card-clickable': clickable }"
    :style="color ? { '--stat-color': color } : {}"
    @click="handleClick"
  >
    <div class="stat-card-header">
      <div v-if="icon" class="stat-card-icon" :style="color ? { color } : {}">
        <component :is="icon" :size="20" />
      </div>
      <span class="stat-card-title">{{ title }}</span>
    </div>

    <div class="stat-card-value" :style="color ? { color } : {}">
      {{ value }}
    </div>

    <div v-if="trendValue" class="stat-card-trend">
      <component :is="trendIcon" :size="14" :style="{ color: trendColor }" />
      <span :style="{ color: trendColor }">{{ trendValue }}</span>
    </div>

    <div v-if="$slots.default" class="stat-card-content">
      <slot />
    </div>

    <div v-if="$slots.footer" class="stat-card-footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<style scoped>
.stat-card {
  background: var(--bg-card);
  border-radius: 12px;
  border: 1px solid var(--border);
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  transition: all 0.2s;
}

.stat-card-clickable {
  cursor: pointer;
}

.stat-card-clickable:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-color: var(--stat-color, var(--accent));
}

.stat-card-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.stat-card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: rgba(108, 99, 255, 0.1);
  color: var(--accent);
}

.stat-card-title {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.stat-card-value {
  font-size: 2rem;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-card-trend {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
}

.stat-card-content {
  font-size: 0.875rem;
  color: var(--text-primary);
}

.stat-card-footer {
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
  font-size: 0.75rem;
  color: var(--text-secondary);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .stat-card {
    padding: 1rem;
  }

  .stat-card-value {
    font-size: 1.75rem;
  }
}
</style>