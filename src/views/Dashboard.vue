<!--
  Dashboard.vue - 仪表盘页面（现代化总览）

  功能：
  - 模块总览：以卡片形式展示所有一级页面（软件/备份/缓存/代理/枚举值/日志）
    的实时统计，点击直接进入对应页面（主窗口路由或独立弹窗）
  - 软件模块使用分段进度条展示「已最新 / 有更新」占比，而非单一环状图

  数据来源：
  - packages store: 软件包列表（总数、已最新、有更新）
  - list_backup_software / list_cache_software: 备份与缓存条目
  - get_proxies: 代理源列表（含 is_active）
  - get_licenses / get_languages: 枚举值管理
  使用组件：
  - StandardizedButton: 操作按钮
  - PageToolbar: 页面工具栏
-->
<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useTabStore } from "../stores/tabs";
import { usePackageStore } from "../stores/packages";
import { invoke } from "@tauri-apps/api/core";
import type {
  ProxyInfo,
  EnumLicense,
  EnumProgrammingLanguage,
  CacheSoftwareEntry,
  BackupSoftwareEntry,
} from "../types";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedButton from "../components/base/StandardizedButton.vue";
import { Icon } from "../icons";
import { openPopup } from "../composables/usePopupWindow";
import type { Component } from "vue";


const router = useRouter();
const tabStore = useTabStore();
const pkgStore = usePackageStore();

// ===== 统计数据 =====
const stats = reactive({
  pkgTotal: 0,
  pkgUpdated: 0,
  pkgOutdated: 0,
  backup: 0,
  cache: 0,
  proxyTotal: 0,
  proxyActive: 0,
  licenses: 0,
  languages: 0,
});
const loading = ref(false);

/** 软件更新占比（用于分段进度条），整数百分比 */
const updatedPct = computed(() =>
  stats.pkgTotal > 0
    ? Math.round((stats.pkgUpdated / stats.pkgTotal) * 100)
    : 0,
);
const outdatedPct = computed(() => 100 - updatedPct.value);

// ===== 导航 =====
/** 跳转到主窗口路由页面（同步打开标签页） */
function goMain(path: string, label: string, icon: Component) {
  tabStore.openTab({ path, label, icon });
  router.push(path);
}

/** 打开独立弹窗页面 */
function goPopup(label: string, url: string, title: string) {
  openPopup(label, url, title);
}

// ===== 模块总览配置 =====
interface ModuleStat {
  label: string;
  value: number | string;
}
interface BarSegment {
  pct: number;
  color: string;
  label: string;
}
interface DashboardModule {
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

const modules = computed<DashboardModule[]>(() => [
  {
    id: "packages",
    title: "软件管理",
    desc: "AUR 软件包的上游版本检查与同步",
    icon: Icon.navPackages,
    color: "var(--accent)",
    stats: [
      { label: "软件包", value: stats.pkgTotal },
      { label: "已最新", value: stats.pkgUpdated },
      { label: "有更新", value: stats.pkgOutdated },
    ],
    actionLabel: "进入软件管理",
    action: () => goMain("/packages", "软件管理", Icon.navPackages),
    bar: [
      { pct: updatedPct.value, color: "var(--success)", label: "已最新" },
      { pct: outdatedPct.value, color: "var(--warning)", label: "有更新" },
    ],
  },
  {
    id: "backup",
    title: "备份管理",
    desc: "本地备份包的安装、去重与恢复",
    icon: Icon.navBackup,
    color: "var(--info)",
    stats: [{ label: "备份记录", value: stats.backup }],
    actionLabel: "进入备份管理",
    action: () => goMain("/backup", "备份管理", Icon.navBackup),
  },
  {
    id: "cache",
    title: "缓存管理",
    desc: "pacman 缓存包与自定义缓存目录",
    icon: Icon.navCache,
    color: "var(--text-secondary)",
    stats: [{ label: "缓存文件", value: stats.cache }],
    actionLabel: "进入缓存管理",
    action: () => goMain("/cache", "缓存管理", Icon.navCache),
  },
  {
    id: "proxy",
    title: "代理管理",
    desc: "代理源的测试、启用与解析",
    icon: Icon.navProxy,
    color: "var(--success)",
    stats: [
      { label: "代理源", value: stats.proxyTotal },
      { label: "可用", value: stats.proxyActive },
    ],
    actionLabel: "进入代理管理",
    action: () => goMain("/proxy", "代理管理", Icon.navProxy),
  },
  {
    id: "enums",
    title: "枚举值管理",
    desc: "License 与编程语言等枚举维护",
    icon: Icon.menuEnums,
    color: "var(--warning)",
    stats: [
      { label: "License", value: stats.licenses },
      { label: "编程语言", value: stats.languages },
    ],
    actionLabel: "打开枚举值管理",
    action: () => goPopup("enums", "/enums", "枚举值管理"),
  },
]);

// ===== 数据加载 =====
async function loadAll() {
  loading.value = true;
  try {
    await pkgStore.fetchPackages();
    stats.pkgTotal = pkgStore.packages.length;
    stats.pkgUpdated = pkgStore.packages.filter((p) => !p.is_outdated).length;
    stats.pkgOutdated = pkgStore.packages.filter((p) => p.is_outdated).length;
  } catch {
    /* 忽略软件包获取错误 */
  }
  try {
    const backups = await invoke<BackupSoftwareEntry[]>("list_backup_software");
    stats.backup = backups.length;
  } catch {
    /* 忽略 */
  }
  try {
    const caches = await invoke<CacheSoftwareEntry[]>("list_cache_software");
    stats.cache = caches.length;
  } catch {
    /* 忽略 */
  }
  try {
    const proxies = await invoke<ProxyInfo[]>("get_proxies");
    stats.proxyTotal = proxies.length;
    stats.proxyActive = proxies.filter((p) => p.is_active).length;
  } catch {
    /* 忽略 */
  }
  try {
    const licenses = await invoke<EnumLicense[]>("get_licenses");
    stats.licenses = licenses.length;
  } catch {
    /* 忽略 */
  }
  try {
    const langs = await invoke<EnumProgrammingLanguage[]>("get_languages");
    stats.languages = langs.length;
  } catch {
    /* 忽略 */
  }
  loading.value = false;
}

onMounted(loadAll);
</script>

<template>
  <div class="dashboard">
    <PageToolbar @refresh="loadAll" />

    <!-- 模块总览 -->
    <section class="module-section">
      <h2 class="section-title">模块总览</h2>
      <div class="module-grid">
        <article
          v-for="m in modules"
          :key="m.id"
          class="module-card"
          :style="{ '--mod-color': m.color }"
        >
          <header class="module-head">
            <div class="module-icon">
              <component :is="m.icon" :size="20" />
            </div>
            <div class="module-title-wrap">
              <h3 class="module-title">{{ m.title }}</h3>
              <p class="module-desc">{{ m.desc }}</p>
            </div>
          </header>

          <!-- 软件模块：分段进度条（替代环状图） -->
          <div v-if="m.bar" class="module-bar-wrap">
            <div class="module-bar">
              <div
                v-for="seg in m.bar"
                :key="seg.label"
                class="module-bar-seg"
                :style="{ width: seg.pct + '%', background: seg.color }"
                :title="`${seg.label} ${seg.pct}%`"
              ></div>
            </div>
            <div class="module-bar-legend">
              <span
                v-for="seg in m.bar"
                :key="seg.label"
                class="legend-item"
              >
                <i class="legend-dot" :style="{ background: seg.color }"></i>
                {{ seg.label }} {{ seg.pct }}%
              </span>
            </div>
          </div>

          <!-- 统计指标 -->
          <ul class="module-stats">
            <li v-for="s in m.stats" :key="s.label" class="module-stat">
              <span class="module-stat-value">{{ s.value }}</span>
              <span class="module-stat-label">{{ s.label }}</span>
            </li>
          </ul>

          <footer class="module-foot">
            <StandardizedButton
              variant="ghost"
              size="sm"
              :style="{ '--bc': m.color }"
              @click="m.action"
            >
              {{ m.actionLabel }}
              <template #icon><component :is="Icon.arrowRight" :size="16" /></template>
            </StandardizedButton>
          </footer>
        </article>
      </div>
    </section>

  </div>
</template>

<style scoped>
.dashboard {
  height: 100vh;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
}

/* ===== 区块 ===== */
.module-section {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* ===== 区块标题 ===== */
.section-title {
  flex: 0 0 auto;
  margin: 0 0 0.5rem;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

/* ===== 模块卡片网格 ===== */
.module-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
  min-height: 0;
  align-content: start;
}

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
.module-enter {
  margin-left: auto;
  color: var(--text-muted);
  flex-shrink: 0;
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

/* ===== 响应式 ===== */
@media (max-width: 1100px) {
  .module-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
@media (max-width: 640px) {
  .module-grid {
    grid-template-columns: 1fr;
  }
}
</style>
