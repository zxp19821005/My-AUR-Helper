<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, inject } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { usePackageStore } from "../stores/packages";
import { FOOTER_KEY } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";
import SoftwareFormModal from "../components/SoftwareFormModal.vue";
import SoftwareDetailModal from "../components/SoftwareDetailModal.vue";
import type { SoftwareListEntry } from "../types";
import {
  RefreshCw,
  Plus,
  Trash2,
  Eye,
  Pencil,
  Info,
  Search,
  Download,
} from "@lucide/vue";

const router = useRouter();
const pkgStore = usePackageStore();
const footer = inject(FOOTER_KEY)!;

const pageSize = 50;
const currentPage = ref(1);
const entries = ref<SoftwareListEntry[]>([]);
const selectedPkgnames = ref(new Set<string>());
const loading = ref(false);

// 弹窗状态
const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalPkgname = ref("");
const showDetailModal = ref(false);
const detailPkgname = ref("");

onMounted(async () => {
  await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  syncToolbar();
});

async function fetchView() {
  loading.value = true;
  try {
    entries.value = await invoke<SoftwareListEntry[]>("list_software_view");
  } finally {
    loading.value = false;
  }
}

const totalRecords = computed(() => entries.value.length);

const pageData = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return entries.value.slice(start, start + pageSize);
});

function goToPage(page: number) {
  currentPage.value = page;
}

function syncToolbar() {
  const s = entries.value;
  const outdated = s.filter((x) => x.is_outdated).length;
  footer.infoText = `总计: ${s.length}  |  已最新: ${s.length - outdated}  |  需更新: ${outdated}`;
  footer.showPagination = s.length > pageSize;
  footer.totalRecords = s.length;
  footer.currentPage = currentPage.value;
  footer.pageSize = pageSize;
  footer.onPageChange = goToPage;
}

watch(totalRecords, syncToolbar);
watch(currentPage, (p) => {
  footer.currentPage = p;
  footer.onPageChange = goToPage;
});

function toggleSelect(pkgname: string) {
  const s = new Set(selectedPkgnames.value);
  if (s.has(pkgname)) s.delete(pkgname);
  else s.add(pkgname);
  selectedPkgnames.value = s;
}

function toggleSelectAll() {
  if (pageData.value.every((p) => selectedPkgnames.value.has(p.pkgname))) {
    selectedPkgnames.value = new Set();
  } else {
    selectedPkgnames.value = new Set(pageData.value.map((p) => p.pkgname));
  }
}

/** 格式化 Unix 时间戳为日期字符串 */
function fmtTimestamp(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit",
  });
}

/** 格式化 ISO 日期字符串 */
function fmtDate(iso: string | null): string {
  if (!iso) return "-";
  const d = new Date(iso);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit",
  });
}

const checkerText: Record<string, string> = {
  github_release: "GitHub Release",
  github_tag: "GitHub Tag",
  gitee: "Gitee",
  gitlab: "GitLab",
  redirect: "重定向",
  http: "HTTP",
  manual: "手动",
};

// ── Toolbar operations ──

async function syncFromAur() {
  loading.value = true;
  try {
    const list = Array.from(selectedPkgnames.value);
    if (list.length) {
      await invoke("update_aur_info", { pkgnameList: list });
    } else {
      await invoke("sync_from_aur");
    }
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

let unlistenProgress: (() => void) | null = null;

async function syncFromPkgbuild() {
  loading.value = true;
  footer.progress = { current: 0, total: 1, message: "准备中..." };
  try {
    // 监听进度事件
    unlistenProgress = await listen<{ current: number; total: number; pkgname: string; message: string }>(
      "sync-progress",
      (event) => {
        const { current, total, message } = event.payload;
        footer.progress = { current, total, message };
      }
    );

    const list = Array.from(selectedPkgnames.value);
    if (list.length) {
      for (const pkgname of list) {
        await invoke("sync_from_pkgbuild", { pkgname });
      }
    } else {
      await invoke("sync_from_pkgbuild", { pkgname: null });
    }
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    unlistenProgress?.();
    unlistenProgress = null;
    footer.progress = null;
    loading.value = false;
  }
}

function openAddModal() {
  modalMode.value = "add";
  modalPkgname.value = "";
  showModal.value = true;
}

function openEditModal(pkgname: string) {
  modalMode.value = "edit";
  modalPkgname.value = pkgname;
  showModal.value = true;
}

function openDetailModal(pkgname: string) {
  detailPkgname.value = pkgname;
  showDetailModal.value = true;
}

function onModalSaved() {
  Promise.all([fetchView(), pkgStore.fetchPackages()]);
}

async function updateAurInfo() {
  loading.value = true;
  try {
    const list = Array.from(selectedPkgnames.value);
    await invoke("update_aur_info", { pkgnameList: list.length ? list : null });
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function checkSelectedUpstream() {
  loading.value = true;
  try {
    const list = Array.from(selectedPkgnames.value);
    if (list.length) {
      await invoke("check_selected_upstream", { pkgnameList: list });
    } else {
      await invoke("check_all_upstream");
    }
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function deleteSelected() {
  const list = Array.from(selectedPkgnames.value);
  if (!list.length) return;
  if (!confirm(`确认删除选中的 ${list.length} 个软件包？`)) return;
  loading.value = true;
  try {
    await invoke("batch_delete_software", { pkgnameList: list });
    selectedPkgnames.value = new Set();
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function checkAll() {
  loading.value = true;
  footer.progress = { current: 0, total: 1 };
  try {
    await invoke("check_all_upstream");
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
    footer.progress = null;
  }
}

// ── Row operations ──

async function rowSyncFromAur(pkgname: string) {
  loading.value = true;
  try {
    await invoke("update_aur_info", { pkgnameList: [pkgname] });
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function rowSyncFromPkgbuild(pkgname: string) {
  loading.value = true;
  try {
    await invoke("sync_from_pkgbuild", { pkgname });
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function rowCheckUpstream(pkgname: string) {
  loading.value = true;
  try {
    await invoke("check_selected_upstream", { pkgnameList: [pkgname] });
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}

async function rowDelete(pkgname: string) {
  if (!confirm(`确认删除 ${pkgname}？`)) return;
  loading.value = true;
  try {
    await invoke("batch_delete_software", { pkgnameList: [pkgname] });
    selectedPkgnames.value = new Set(Array.from(selectedPkgnames.value).filter(n => n !== pkgname));
    await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn-icon btn-icon-accent" @click="syncFromAur" :disabled="loading" title="从AUR同步">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="syncFromPkgbuild" :disabled="loading" title="从PKGBUILD同步">
        <Download :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" @click="openAddModal" title="添加软件">
        <Plus :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="updateAurInfo" :disabled="loading" title="更新AUR信息">
        <Info :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="checkSelectedUpstream" :disabled="loading || selectedPkgnames.size === 0" title="更新上游信息">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" @click="deleteSelected" :disabled="selectedPkgnames.size === 0" title="删除选中">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-warning" @click="checkAll" :disabled="loading" title="检查全部更新">
        <Search :size="16" />
      </button>
    </PageToolbar>

    <div class="card" style="overflow-x: auto">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input type="checkbox"
                :checked="pageData.length > 0 && pageData.every(p => selectedPkgnames.has(p.pkgname))"
                :indeterminate="pageData.some(p => selectedPkgnames.has(p.pkgname)) && !pageData.every(p => selectedPkgnames.has(p.pkgname))"
                @change="toggleSelectAll" />
            </th>
            <th>包名</th>
            <th>AUR 版本</th>
            <th>AUR 最后提交</th>
            <th>上游版本</th>
            <th>上游检查日期</th>
            <th style="min-width: 200px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pkg in pageData" :key="pkg.pkgname"
            :class="{ 'row-selected': selectedPkgnames.has(pkg.pkgname) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedPkgnames.has(pkg.pkgname)"
                @change="toggleSelect(pkg.pkgname)" />
            </td>
            <td>
              <strong>{{ pkg.pkgname }}</strong>
              <span v-if="pkg.is_outdated" class="status-badge status-update_available" style="margin-left: 0.5rem">需更新</span>
            </td>
            <td>{{ pkg.aur_version || "-" }}</td>
            <td>{{ fmtTimestamp(pkg.aur_last_updated) }}</td>
            <td>{{ pkg.upstream_version || "-" }}</td>
            <td>{{ fmtDate(pkg.upstream_last_checked) }}</td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-default" @click.stop="openDetailModal(pkg.pkgname)" title="查看详情">
                  <Eye :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="openEditModal(pkg.pkgname)" title="软件编辑">
                  <Pencil :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromAur(pkg.pkgname)" :disabled="loading" title="从AUR同步">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromPkgbuild(pkg.pkgname)" :disabled="loading" title="从PKGBUILD同步">
                  <Download :size="14" />
                </button>
                <button class="btn-icon btn-icon-info" @click.stop="rowCheckUpstream(pkg.pkgname)" :disabled="loading" title="更新上游信息">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.pkgname)" :disabled="loading" title="删除">
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <SoftwareFormModal
      :show="showModal"
      :mode="modalMode"
      :pkgname="modalPkgname"
      @close="showModal = false"
      @saved="onModalSaved"
    />

    <SoftwareDetailModal
      :show="showDetailModal"
      :pkgname="detailPkgname"
      @close="showDetailModal = false"
    />
  </div>
</template>

<style scoped>
.pkg-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: auto;
}
.pkg-table th {
  text-align: left;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.pkg-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}
.pkg-table tbody tr {
  cursor: pointer;
  transition: background-color 0.15s;
}
.pkg-table tbody tr:hover {
  background-color: rgba(108, 99, 255, 0.05);
}
.pkg-table tbody tr.row-selected {
  background-color: rgba(108, 99, 255, 0.1);
}

.row-actions {
  display: flex;
  gap: 0.25rem;
  flex-wrap: nowrap;
  align-items: center;
}

/* 工具栏按钮图标样式 */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}
.btn svg {
  flex-shrink: 0;
}

/* 操作列图标按钮 */
.btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}
.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* 默认 - 查看/编辑 */
.btn-icon-default {
  background-color: rgba(154, 156, 184, 0.12);
  color: var(--text-secondary);
}
.btn-icon-default:hover:not(:disabled) {
  background-color: rgba(154, 156, 184, 0.25);
  color: var(--text-primary);
}

/* 同步/操作 - 强调色 */
.btn-icon-accent {
  background-color: rgba(108, 99, 255, 0.1);
  color: var(--accent);
}
.btn-icon-accent:hover:not(:disabled) {
  background-color: rgba(108, 99, 255, 0.2);
  color: var(--accent);
}

/* 添加 - 成功色 */
.btn-icon-success {
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
}
.btn-icon-success:hover:not(:disabled) {
  background-color: rgba(76, 175, 125, 0.2);
  color: var(--success);
}

/* 信息更新 - 信息色 */
.btn-icon-info {
  background-color: rgba(66, 165, 245, 0.1);
  color: #42a5f5;
}
.btn-icon-info:hover:not(:disabled) {
  background-color: rgba(66, 165, 245, 0.2);
  color: #42a5f5;
}

/* 删除 - 危险色 */
.btn-icon-danger {
  background-color: rgba(239, 83, 80, 0.1);
  color: var(--error);
}
.btn-icon-danger:hover:not(:disabled) {
  background-color: rgba(239, 83, 80, 0.2);
  color: var(--error);
}

/* 检查 - 警告色 */
.btn-icon-warning {
  background-color: rgba(255, 167, 38, 0.1);
  color: var(--warning);
}
.btn-icon-warning:hover:not(:disabled) {
  background-color: rgba(255, 167, 38, 0.2);
  color: var(--warning);
}

.btn-sm {
  padding: 0.2rem 0.5rem;
  font-size: 0.75rem;
}
.btn-danger-text {
  color: var(--danger, #e53e3e);
}
.btn-danger-text:hover {
  background-color: var(--danger, #e53e3e);
  color: #fff;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal {
  background: var(--bg-card, #fff);
  border-radius: 8px;
  padding: 1.5rem;
  min-width: 320px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
}
.modal h3 {
  margin: 0 0 1rem;
}
.form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.875rem;
  box-sizing: border-box;
}
.modal-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
  margin-top: 1rem;
}
</style>
