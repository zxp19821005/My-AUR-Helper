<script setup lang="ts">
import { computed, onMounted, ref, watch, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { FOOTER_KEY } from "../composables/footer";
import { usePackageActions } from "../composables/packageActions";
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
  Download,
} from "@lucide/vue";

const pkgStore = usePackageStore();
const footer = inject(FOOTER_KEY)!;

const pageSize = 50;
const currentPage = ref(1);
const entries = ref<SoftwareListEntry[]>([]);
const selectedPkgnames = ref(new Set<string>());

// 弹窗状态
const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalPkgname = ref("");
const showDetailModal = ref(false);
const detailPkgname = ref("");

const {
  loading,
  isRowLoading,
  syncFromAur,
  syncFromPkgbuild,
  updateAurInfo,
  checkSelectedUpstream,
  deleteSelected,
  rowSyncFromAur,
  rowSyncFromPkgbuild,
  rowCheckUpstream,
  rowDelete,
} = usePackageActions(fetchView, footer);

onMounted(async () => {
  await Promise.all([fetchView(), pkgStore.fetchPackages()]);
  syncToolbar();
});

async function fetchView() {
  try {
    entries.value = await invoke<SoftwareListEntry[]>("list_software_view");
  } finally {
    syncToolbar();
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

function fmtTimestamp(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
}

function fmtDate(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
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

const setSelected = (v: Set<string>) => { selectedPkgnames.value = v; };
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn-icon btn-icon-accent" @click="syncFromAur(selectedPkgnames)" :disabled="loading" title="从AUR同步">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="syncFromPkgbuild(selectedPkgnames)" :disabled="loading" title="从PKGBUILD同步">
        <Download :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" @click="openAddModal" title="添加软件">
        <Plus :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="updateAurInfo(selectedPkgnames)" :disabled="loading" title="更新AUR信息">
        <Info :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="checkSelectedUpstream(selectedPkgnames)" :disabled="loading" title="更新上游信息">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" @click="deleteSelected(selectedPkgnames, setSelected)" :disabled="selectedPkgnames.size === 0" title="删除选中">
        <Trash2 :size="16" />
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
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromAur(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname)" title="从AUR同步">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromPkgbuild(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname)" title="从PKGBUILD同步">
                  <Download :size="14" />
                </button>
                <button class="btn-icon btn-icon-info" @click.stop="rowCheckUpstream(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname)" title="更新上游信息">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.pkgname, selectedPkgnames, setSelected)" :disabled="isRowLoading(pkg.pkgname)" title="删除">
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
      @navigate="detailPkgname = $event"
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

</style>