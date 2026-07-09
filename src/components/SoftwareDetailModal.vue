<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareDetail } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
import Modal from "./common/Modal.vue";
import SoftwareFormModal from "./SoftwareFormModal.vue";
import FloatingNav from "./FloatingNav.vue";
import DetailToolbar from "./DetailToolbar.vue";

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
    const [prev, next] = await invoke<[string | null, string | null]>("get_prev_next_software", { pkgname: props.pkgname });
    prevPkgname.value = prev;
    nextPkgname.value = next;
  } catch { /* ignore */ }
}

function navigate(direction: "prev" | "next") {
  const target = direction === "prev" ? prevPkgname.value : nextPkgname.value;
  if (target) emit("navigate", target);
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
  return ts ? new Date(ts * 1000).toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  }) : "—";
}

function getPkgTypeName(id: number | null): string {
  return pkgTypeOptions.find(t => t.id === id)?.label || '未知';
}

function getCheckerTypeName(id: number | null): string {
  return checkerTypeOptions.find(c => c.id === id)?.label || '未知';
}

watch(() => props.pkgname, () => { if (props.show) loadSoftware(); });
watch(() => props.show, (val) => { if (!val) showEditModal.value = false; });
</script>

<template>
  <Modal :show="show" width="640px" hide-header @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>

    <div class="detail-header">
      <h3 class="pkg-title">{{ detail?.pkgname || "软件详情" }}</h3>
    </div>

    <FloatingNav :prev="prevPkgname" :next="nextPkgname" @navigate="navigate" />

    <div v-if="loading" class="loading-text">加载中...</div>

    <div v-else-if="detail" class="detail-content">
      <div class="section">
        <h4 class="section-title">基本信息</h4>
        <div class="badge-row">
          <span :class="['status-badge', detail.is_outdated ? 'outdated' : 'latest']">
            {{ detail.is_outdated ? '需更新' : '已最新' }}
          </span>
          <span class="type-tag">{{ getPkgTypeName(detail.package_type_id) }}</span>
          <span class="type-tag">{{ getCheckerTypeName(detail.checker_type_id) }}</span>
        </div>
        <table class="info-table">
          <tbody>
            <tr><td class="label">上游地址</td><td class="value url-cell">
              <a v-if="detail.upstream_url" :href="detail.upstream_url" target="_blank">{{ detail.upstream_url }}</a>
              <span v-else>未设置</span></td></tr>
            <tr><td class="label">包描述</td><td class="value">{{ detail.aur_pkgdesc || '—' }}</td></tr>
            <tr><td class="label">版本提取关键字</td><td class="value">
              <code v-if="detail.version_extract_regex">{{ detail.version_extract_regex }}</code>
              <span v-else class="empty">未设置</span></td></tr>
          </tbody>
        </table>
        <div class="status-row">
          <span class="status-item">
            <span class="status-label">自动检查</span>
            <span :class="['status-value', detail.auto_check_enabled ? 'enabled' : 'disabled']">
              {{ detail.auto_check_enabled ? '已启用' : '已禁用' }}
            </span>
          </span>
          <span class="status-item">
            <span class="status-label">测试版本</span>
            <span :class="['status-value', detail.check_test_versions ? 'enabled' : 'disabled']">
              {{ detail.check_test_versions ? '已启用' : '已禁用' }}
            </span>
          </span>
          <span class="status-item">
            <span class="status-label">二进制文件</span>
            <span :class="['status-value', detail.check_binary_files ? 'enabled' : 'disabled']">
              {{ detail.check_binary_files ? '已启用' : '已禁用' }}
            </span>
          </span>
        </div>
      </div>

      <div class="side-by-side">
        <div class="section half-section">
          <h4 class="section-title">AUR 信息</h4>
          <table class="info-table">
            <tbody>
              <tr><td class="label">AUR 版本</td><td class="value version-cell">{{ detail.aur_version || '—' }}</td></tr>
              <tr><td class="label">更新时间</td><td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td></tr>
            </tbody>
          </table>
        </div>

        <div class="section half-section">
          <h4 class="section-title">上游版本信息</h4>
          <table class="info-table">
            <tbody>
              <tr><td class="label">上游版本</td><td class="value version-cell">{{ detail.upstream_version || '—' }}</td></tr>
              <tr><td class="label">上次检查</td><td class="value">{{ formatTimestamp(detail.upstream_last_checked) }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <template #footer>
      <DetailToolbar
        :loading="loading"
        :updating-aur="updatingAur"
        :updating-pkgbuild="updatingPkgbuild"
        :checking="checking"
        :deleting="deleting"
        @edit="showEditModal = true"
        @delete="handleDelete"
        @update-aur="updateAurInfo"
        @update-pkgbuild="updatePkgbuild"
        @check-update="checkUpdate"
      />
    </template>
  </Modal>

  <SoftwareFormModal
    :show="showEditModal"
    mode="edit"
    :pkgname="detail?.pkgname"
    @close="showEditModal = false"
    @saved="loadSoftware"
  />
</template>

<style scoped>
.detail-header { text-align: center; margin-bottom: 0.75rem; }
.pkg-title { font-size: 0.9375rem; font-weight: 600; color: var(--text-primary); margin: 0; }

.loading-text { color: var(--text-secondary); text-align: center; padding: 1.5rem 0; }
.detail-content { max-height: 50vh; overflow-y: auto; }

.side-by-side { display: flex; gap: 1rem; }
.side-by-side .half-section { flex: 1; min-width: 0; }

.status-row {
  display: flex;
  gap: 1.25rem;
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.status-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.status-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.status-value {
  font-size: 0.75rem;
  font-weight: 500;
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
}

.status-value.enabled {
  color: var(--success);
  background: var(--success-bg);
}

.status-value.disabled {
  color: var(--error);
  background: var(--error-bg);
}
</style>
