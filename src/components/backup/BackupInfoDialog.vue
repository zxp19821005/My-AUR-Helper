<!--
  BackupInfoDialog.vue - 备份包详情弹窗

  功能：
  - 紧凑双行展示备份包元数据（基本信息+文件信息合并，无卡片标题）
  - 展示 pacman -Qip 原始输出（详细信息）
  - 内容区右侧浮动上一个/下一个导航按钮（与软件详情页一致）
  - 底部工具栏：仅安装/删除（纯图标方形按钮）
-->
<script setup lang="ts">
import { computed } from "vue";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedCard from "../base/StandardizedCard.vue";
import StandardizedBadge from "../base/StandardizedBadge.vue";
import StandardizedButton from "../base/StandardizedButton.vue";
import NavPager from "../common/NavPager.vue";
import { Icon } from "../../icons";
import type { BackupSoftwareEntry } from "../../types";

const props = defineProps<{
  /** 是否显示弹窗 */
  show: boolean;
  /** 原始信息加载状态 */
  loading: boolean;
  /** 包名（兜底标题，优先使用 entry.pkgname） */
  pkgname: string;
  /** pacman -Qip 原始输出 */
  content: string;
  /** 完整备份记录（用于展示与操作） */
  entry: BackupSoftwareEntry | null;
  /** 上一个备份条目（null 表示不可用） */
  prevEntry: BackupSoftwareEntry | null;
  /** 下一个备份条目（null 表示不可用） */
  nextEntry: BackupSoftwareEntry | null;
}>();

const emit = defineEmits<{
  close: [];
  install: [entry: BackupSoftwareEntry];
  delete: [entry: BackupSoftwareEntry];
  navigate: [entry: BackupSoftwareEntry];
}>();

/** 安装中状态（由父组件透传） */
const installing = defineModel<boolean>("installing", { default: false });
/** 删除中状态（由父组件透传） */
const deleting = defineModel<boolean>("deleting", { default: false });

/** 完整版本号 */
const fullVersion = computed(() => {
  if (!props.entry) return "";
  const prefix = props.entry.epoch > 0 ? `${props.entry.epoch}:` : "";
  return `${prefix}${props.entry.pkgver}-${props.entry.pkgrel}`;
});

function onInstall() {
  if (props.entry) emit("install", props.entry);
}

function onDelete() {
  if (props.entry) emit("delete", props.entry);
}

function onNavigate(direction: "prev" | "next") {
  const target = direction === "prev" ? props.prevEntry : props.nextEntry;
  if (target) emit("navigate", target);
}
</script>

<template>
  <StandardizedModal
    :show="show"
    width="lg"
    @close="emit('close')"
  >
    <template #header>
      <h3 class="modal-title">
        {{ entry?.pkgname || pkgname }}
        <StandardizedBadge
          v-if="entry?.arch"
          type="info"
          :text="entry.arch"
          size="sm"
        />
        <StandardizedBadge
          v-if="entry?.subdirectory"
          type="neutral"
          :text="entry.subdirectory"
          size="sm"
        />
      </h3>
    </template>

    <!-- 主体：紧凑元数据 + 详细信息 + 浮动导航 -->
    <div class="dialog-body">
      <!-- 紧凑元数据区（2行，无卡片标题） -->
      <div v-if="entry" class="meta-compact">
        <div class="meta-row">
          <span class="meta-item"><b>包名</b>{{ entry.pkgname }}</span>
          <span class="meta-item"><b>版本</b>{{ fullVersion }}</span>
          <span class="meta-item"><b>架构</b>{{ entry.arch || "-" }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-item meta-filename"><b>文件</b><code>{{ entry.filename }}</code></span>
          <span class="meta-item"><b>子目录</b>{{ entry.subdirectory || "-" }}</span>
        </div>
      </div>

      <!-- 详细信息卡片 + 悬浮左右导航 -->
      <div class="raw-wrapper">
        <StandardizedCard title="详细信息 (pacman -Qip)" class="raw-card">
          <div v-if="loading" class="loading-spinner">加载中...</div>
          <pre v-else class="raw-content">{{ content || "暂无信息" }}</pre>
        </StandardizedCard>

        <NavPager
          variant="floating"
          :prev="prevEntry?.pkgname ?? null"
          :next="nextEntry?.pkgname ?? null"
          @navigate="onNavigate"
        />
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-spacer"></div>
        <StandardizedButton
          class="tool-btn"
          variant="outline"
          tone="warning"
          :disabled="installing"
          :title="`安装 ${entry?.pkgname || ''}`"
          @click="onInstall"
        >
          <component :is="Icon.install" :size="18" :class="{ spinning: installing }" />
        </StandardizedButton>
        <StandardizedButton
          class="tool-btn"
          variant="outline"
          tone="danger"
          :disabled="deleting"
          :title="`删除 ${entry?.pkgname || ''}`"
          @click="onDelete"
        >
          <component :is="Icon.actionDelete" :size="18" :class="{ spinning: deleting }" />
        </StandardizedButton>
      </div>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.modal-title {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
}

/* ====== 紧凑元数据（2行，无卡片包裹）====== */
.meta-compact {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.625rem 0.875rem;
  margin-bottom: 0.75rem;
}

.meta-row {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  flex-wrap: wrap;
  line-height: 1.6;
}

.meta-row + .meta-row {
  margin-top: 0.25rem;
}

.meta-item {
  font-size: 0.8125rem;
  color: var(--text-primary);
  white-space: nowrap;
}

.meta-item b {
  color: var(--text-secondary);
  margin-right: 0.375rem;
  font-weight: 500;
}

.meta-item code {
  font-family: monospace;
  font-size: 0.75rem;
  background: var(--bg-secondary);
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
  word-break: break-all;
}

.meta-filename {
  max-width: 100%;
  flex: 1;
  min-width: 0;
}

.meta-filename code {
  white-space: normal;
  display: inline;
}

/* ====== 详细信息卡片 + 悬浮左右导航 ====== */
.raw-wrapper {
  position: relative;
}

/* ====== 详细信息卡片 ====== */
.raw-card {
  margin: 0;
}

.raw-card :deep(.card-header) {
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
}

.loading-spinner {
  text-align: center;
  color: var(--text-secondary);
  padding: 1.5rem;
}

.raw-content {
  font-family: monospace;
  font-size: 0.75rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
  background: var(--bg-secondary);
  padding: 0.75rem;
  border-radius: 6px;
  margin: 0;
}

/* ====== 底部工具栏 ====== */
.dialog-footer {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
}

.footer-spacer {
  flex: 1;
}

/* 纯图标方形按钮 */
.dialog-footer .tool-btn {
  justify-content: center;
  gap: 0;
  padding: 0.5rem;
  width: 2.5rem;
  height: 2.5rem;
}

@media (max-width: 768px) {
  .meta-row {
    flex-direction: column;
    gap: 0.375rem;
  }
}
</style>
