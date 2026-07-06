<!--
  SoftwareDetailModal.vue - 软件包详情弹窗

  Props:
  - show: boolean
  - pkgname: string

  Events:
  - close: 关闭弹窗
-->
<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, ArrowRight, Edit, Trash2, RefreshCw, FileCode, GitBranch, X } from "@lucide/vue";
import type { SoftwareDetail } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
import Modal from "./common/Modal.vue";
import SoftwareFormModal from "./SoftwareFormModal.vue";

const props = defineProps<{
  show: boolean;
  pkgname: string;
}>();

const emit = defineEmits<{
  close: [];
  navigate: [pkgname: string];
}>();

const detail = ref<SoftwareDetail | null>(null);
const loading = ref(false);
const error = ref("");
const prevPkgname = ref<string | null>(null);
const nextPkgname = ref<string | null>(null);
const showEditModal = ref(false);

const updatingAur = ref(false);
const updatingPkgbuild = ref(false);
const checking = ref(false);
const deleting = ref(false);

const pkgTypeName = computed(() => {
  return pkgTypeOptions.find(t => t.id === detail.value?.package_type_id)?.label || '未知';
});

const checkerTypeName = computed(() => {
  return checkerTypeOptions.find(c => c.id === detail.value?.checker_type_id)?.label || '未知';
});

async function loadSoftware() {
  if (!props.pkgname) return;
  loading.value = true;
  error.value = "";
  try {
    detail.value = await invoke<SoftwareDetail | null>("get_software_detail", { pkgname: props.pkgname });
    if (!detail.value) error.value = "未找到软件包";
    await loadNav();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function loadNav() {
  try {
    const [prev, next] = await invoke<[string | null, string | null]>("get_prev_next_software", {
      pkgname: props.pkgname,
    });
    prevPkgname.value = prev;
    nextPkgname.value = next;
  } catch {
    // ignore
  }
}

function navigate(direction: "prev" | "next") {
  const target = direction === "prev" ? prevPkgname.value : nextPkgname.value;
  if (target) {
    emit("navigate", target);
  }
}

async function updateAurInfo() {
  if (!detail.value) return;
  updatingAur.value = true;
  error.value = "";
  try {
    await invoke<number>("update_aur_info", { pkgnameList: [detail.value.pkgname] });
    await loadSoftware();
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingAur.value = false;
  }
}

async function updatePkgbuild() {
  if (!detail.value) return;
  updatingPkgbuild.value = true;
  error.value = "";
  try {
    await invoke<number>("sync_from_pkgbuild", { pkgname: detail.value.pkgname });
    await loadSoftware();
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingPkgbuild.value = false;
  }
}

async function checkUpdate() {
  if (!detail.value) return;
  checking.value = true;
  error.value = "";
  try {
    await invoke<string>("check_upstream_version", { pkgname: detail.value.pkgname });
    await loadSoftware();
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
  }
}

async function handleDelete() {
  if (!detail.value?.software_id) return;
  if (!confirm(`确定要删除软件包 "${detail.value.pkgname}" 吗？`)) return;

  deleting.value = true;
  error.value = "";
  try {
    await invoke("delete_software", { softwareId: detail.value.software_id });
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    deleting.value = false;
  }
}

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString("zh-CN");
}

watch(
  () => props.pkgname,
  () => {
    if (props.show) {
      loadSoftware();
    }
  }
);

watch(
  () => props.show,
  (val) => {
    if (!val) {
      showEditModal.value = false;
    }
  }
);
</script>

<template>
  <Modal :show="show" width="640px" hide-header @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>

    <!-- 弹窗头部：标题 + 居中浮动导航按钮 -->
    <div class="detail-header">
      <div class="header-nav">
        <button
          class="nav-btn"
          :class="{ disabled: !prevPkgname }"
          @click.stop="navigate('prev')"
          title="上一个"
        >
          <ArrowLeft :size="18" />
        </button>
        <span class="pkg-title">{{ detail?.pkgname || "软件详情" }}</span>
        <button
          class="nav-btn"
          :class="{ disabled: !nextPkgname }"
          @click.stop="navigate('next')"
          title="下一个"
        >
          <ArrowRight :size="18" />
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading-text">加载中...</div>

    <!-- 内容区域 -->
    <div v-else-if="detail" class="detail-content">
      <!-- 基本信息 -->
      <div class="section">
        <h4 class="section-title">基本信息</h4>
        <div class="badge-row">
          <span :class="['status-badge', detail.is_outdated ? 'outdated' : 'latest']">
            {{ detail.is_outdated ? '需更新' : '已最新' }}
          </span>
          <span class="type-tag">{{ pkgTypeName }}</span>
          <span class="type-tag">{{ checkerTypeName }}</span>
        </div>
        <table class="info-table">
          <tbody>
            <tr>
              <td class="label">上游地址</td>
              <td class="value url-cell">
                <a v-if="detail.upstream_url" :href="detail.upstream_url" target="_blank">
                  {{ detail.upstream_url }}
                </a>
                <span v-else>未设置</span>
              </td>
            </tr>
            <tr>
              <td class="label">包描述</td>
              <td class="value">{{ detail.aur_pkgdesc || '—' }}</td>
            </tr>
            <tr>
              <td class="label">自动检查</td>
              <td class="value">{{ detail.auto_check_enabled ? '已启用' : '已禁用' }}</td>
            </tr>
            <tr>
              <td class="label">测试版本</td>
              <td class="value">{{ detail.check_test_versions ? '已启用' : '已禁用' }}</td>
            </tr>
            <tr>
              <td class="label">二进制文件</td>
              <td class="value">{{ detail.check_binary_files ? '已启用' : '已禁用' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- AUR 信息和上游信息并排 -->
      <div class="side-by-side">
        <!-- AUR 信息 -->
        <div class="section half-section">
          <h4 class="section-title">AUR 信息</h4>
          <table class="info-table">
            <tbody>
              <tr>
                <td class="label">AUR 版本</td>
                <td class="value version-cell">{{ detail.aur_version || '—' }}</td>
              </tr>
              <tr>
                <td class="label">更新时间</td>
                <td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 上游信息 -->
        <div class="section half-section">
          <h4 class="section-title">上游版本信息</h4>
          <table class="info-table">
            <tbody>
              <tr>
                <td class="label">上游版本</td>
                <td class="value version-cell">{{ detail.upstream_version || '—' }}</td>
              </tr>
              <tr>
                <td class="label">上次检查</td>
                <td class="value">{{ detail.upstream_last_checked || '—' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- 底部工具栏 -->
    <template #footer>
      <div class="footer-toolbar">
        <div class="footer-left">
          <button class="footer-btn" @click="showEditModal = true" :disabled="loading">
            <Edit :size="14" />
            <span>编辑</span>
          </button>
          <button class="footer-btn danger" @click="handleDelete" :disabled="deleting || loading">
            <Trash2 :size="14" />
            <span>删除</span>
          </button>
        </div>
        <div class="footer-right">
          <button class="footer-btn" @click="updateAurInfo" :disabled="updatingAur || loading">
            <GitBranch :size="14" />
            <span>{{ updatingAur ? '更新中...' : 'AUR 信息' }}</span>
          </button>
          <button class="footer-btn" @click="updatePkgbuild" :disabled="updatingPkgbuild || loading">
            <FileCode :size="14" />
            <span>{{ updatingPkgbuild ? '更新中...' : 'PKGBUILD' }}</span>
          </button>
          <button class="footer-btn primary" @click="checkUpdate" :disabled="checking || loading">
            <RefreshCw :size="14" :class="{ spinning: checking }" />
            <span>{{ checking ? '检查中...' : '检查上游' }}</span>
          </button>
        </div>
      </div>
    </template>
  </Modal>

  <!-- 编辑弹窗 -->
  <SoftwareFormModal
    :show="showEditModal"
    mode="edit"
    :pkgname="detail?.pkgname"
    @close="showEditModal = false"
    @saved="loadSoftware"
  />
</template>

<style scoped>
.detail-header {
  display: flex;
  justify-content: center;
  margin-bottom: 0.75rem;
}

.header-nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.25rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.pkg-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--text-primary);
  padding: 0.375rem 1rem;
  white-space: nowrap;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.nav-btn:hover:not(.disabled) {
  background-color: var(--bg-hover);
  color: var(--accent);
}

.nav-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.loading-text {
  color: var(--text-secondary);
  text-align: center;
  padding: 1.5rem 0;
}

.detail-content {
  max-height: 50vh;
  overflow-y: auto;
}

.side-by-side {
  display: flex;
  gap: 1rem;
}

.side-by-side .half-section {
  flex: 1;
  min-width: 0;
}

.section {
  margin-bottom: 1rem;
}

.section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--accent);
  margin: 0 0 0.5rem 0;
  padding-bottom: 0.25rem;
  border-bottom: 1px solid var(--border);
}

.badge-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.status-badge {
  font-size: 0.75rem;
  font-weight: 500;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
}

.status-badge.latest {
  background-color: rgba(82, 196, 26, 0.15);
  color: #52c41a;
}

.status-badge.outdated {
  background-color: rgba(250, 173, 20, 0.15);
  color: #faad14;
}

.type-tag {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
  background-color: var(--bg-hover);
  color: var(--text-secondary);
}

.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 0.375rem 0;
  font-size: 0.8125rem;
}

.info-table .label {
  width: 100px;
  color: var(--text-secondary);
}

.info-table .value {
  color: var(--text-primary);
}

.url-cell a {
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}

.url-cell a:hover {
  text-decoration: underline;
}

.version-cell {
  font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
  font-weight: 500;
}

.footer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.footer-left,
.footer-right {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: 0.3125rem;
  padding: 0.375rem 0.75rem;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.footer-btn:hover:not(:disabled) {
  border-color: var(--accent);
  background-color: var(--bg-hover);
}

.footer-btn.primary {
  background-color: var(--accent);
  border-color: var(--accent);
  color: white;
}

.footer-btn.primary:hover:not(:disabled) {
  opacity: 0.9;
}

.footer-btn.danger {
  border-color: var(--error);
  color: var(--error);
}

.footer-btn.danger:hover:not(:disabled) {
  background-color: var(--error-bg);
}

.footer-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
