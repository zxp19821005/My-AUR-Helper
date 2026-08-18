<!--
  ModuleCard.vue - 仪表盘模块总览卡片

  功能：渲染「模块总览」中的单个模块卡片（图标、描述、分段进度条、
  统计指标、进入按钮）。数据由父组件 Dashboard 通过 module prop 传入，
  点击进入按钮直接调用 module.action()（闭包已绑定好路由/弹窗跳转）。

  使用组件：
  - StandardizedButton: 操作按钮
-->
<script lang="ts">
// 导出类型供 Dashboard 构造 modules 数组时使用
import type { Component } from "vue";

export interface ModuleStat {
  label: string;
  value: number | string;
}

export interface BarSegment {
  pct: number;
  color: string;
  label: string;
}

export interface DashboardModule {
  id: string;
  title: string;
  desc: string;
  icon: Component;
  color: string;
  stats: ModuleStat[];
  actionLabel: string;
  action: () => void;
  /** 可选：分段进度条（替代环状图） */
  bar?: BarSegment[];
}
</script>

<script setup lang="ts">
import { Icon } from "../../icons";
import StandardizedButton from "../base/StandardizedButton.vue";

defineProps<{ module: DashboardModule }>();
</script>

<template>
  <article class="module-card" :style="{ '--mod-color': module.color }">
    <header class="module-head">
      <div class="module-icon">
        <component :is="module.icon" :size="20" />
      </div>
      <div class="module-title-wrap">
        <h3 class="module-title">{{ module.title }}</h3>
        <p class="module-desc">{{ module.desc }}</p>
      </div>
    </header>

    <!-- 分段进度条（替代环状图） -->
    <div v-if="module.bar" class="module-bar-wrap">
      <div class="module-bar">
        <div
          v-for="seg in module.bar"
          :key="seg.label"
          class="module-bar-seg"
          :style="{ width: seg.pct + '%', background: seg.color }"
          :title="`${seg.label} ${seg.pct}%`"
        ></div>
      </div>
      <div class="module-bar-legend">
        <span v-for="seg in module.bar" :key="seg.label" class="legend-item">
          <i class="legend-dot" :style="{ background: seg.color }"></i>
          {{ seg.label }} {{ seg.pct }}%
        </span>
      </div>
    </div>

    <!-- 统计指标 -->
    <ul class="module-stats">
      <li v-for="s in module.stats" :key="s.label" class="module-stat">
        <span class="module-stat-value">{{ s.value }}</span>
        <span class="module-stat-label">{{ s.label }}</span>
      </li>
    </ul>

    <footer class="module-foot">
      <StandardizedButton
        variant="ghost"
        size="sm"
        :style="{ '--bc': module.color }"
        @click="module.action"
      >
        {{ module.actionLabel }}
        <template #icon><component :is="Icon.arrowRight" :size="16" /></template>
      </StandardizedButton>
    </footer>
  </article>
</template>

<style scoped>
.module-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 0.65rem 0.85rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  min-height: 0;
  overflow: hidden;
  transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
}
.module-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  border-color: var(--mod-color, var(--accent));
}

.module-head {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
}
.module-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  flex-shrink: 0;
  color: var(--mod-color, var(--accent));
  background: color-mix(in srgb, var(--mod-color, var(--accent)) 14%, transparent);
}
.module-icon > :deep(*) {
  width: 18px;
  height: 18px;
}
.module-title-wrap {
  min-width: 0;
}
.module-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-primary);
}
.module-desc {
  margin: 0.15rem 0 0;
  font-size: 0.72rem;
  color: var(--text-muted);
  line-height: 1.35;
}

/* 分段进度条 */
.module-bar-wrap {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.module-bar {
  display: flex;
  height: 6px;
  border-radius: 999px;
  overflow: hidden;
  background: var(--bg-secondary);
}
.module-bar-seg {
  height: 100%;
  transition: width 0.4s ease;
}
.module-bar-seg:first-child {
  border-radius: 999px 0 0 999px;
}
.module-bar-legend {
  display: flex;
  gap: 0.75rem;
  font-size: 0.7rem;
  color: var(--text-secondary);
}
.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}
.legend-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

/* 统计指标 */
.module-stats {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}
.module-stat {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}
.module-stat-value {
  font-size: 1.05rem;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.1;
}
.module-stat-label {
  font-size: 0.7rem;
  color: var(--text-muted);
}

.module-foot {
  margin-top: 0.15rem;
}
</style>
