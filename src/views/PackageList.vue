<script setup lang="ts">
import { computed, onMounted, ref, watch, inject } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { FOOTER_KEY } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";

const router = useRouter();
const pkgStore = usePackageStore();
const footer = inject(FOOTER_KEY)!;

const pageSize = 50;
const currentPage = ref(1);
const selectedPkgnames = ref(new Set<string>());
const loading = ref(false);
const showAddDialog = ref(false);
const addPkgname = ref("");
const addLoading = ref(false);

onMounted(async () => {
  await pkgStore.fetchPackages();
  syncToolbar();
});

const checkerText: Record<string, string> = {
  github_release: "GitHub Release",
  github_tag: "GitHub Tag",
  gitee: "Gitee",
  gitlab: "GitLab",
  redirect: "重定向",
  http: "HTTP",
  manual: "手动",
};

const summary = computed(() => {
  const p = pkgStore.packages;
  return {
    total: p.length,
    outdated: p.filter((x) => x.is_outdated).length,
    upToDate: p.filter((x) => !x.is_outdated).length,
  };
});

const pageData = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return pkgStore.packages.slice(start, start + pageSize);
});

function goToPage(page: number) {
  currentPage.value = page;
}

function syncToolbar() {
  const s = summary.value;
  footer.infoText = `总计: ${s.total}  |  已最新: ${s.upToDate}  |  需更新: ${s.outdated}`;
  footer.showPagination = s.total > pageSize;
  footer.totalRecords = s.total;
  footer.currentPage = currentPage.value;
  footer.pageSize = pageSize;
  footer.onPageChange = goToPage;
}

watch(summary, syncToolbar);
watch(currentPage, (p) => {
  footer.currentPage = p;
  footer.onPageChange = goToPage;
});

function toggleSelect(pkgname: string) {
  const s = new Set(selectedPkgnames.value);
  if (s.has(pkgname)) {
    s.delete(pkgname);
  } else {
    s.add(pkgname);
  }
  selectedPkgnames.value = s;
}

function toggleSelectAll() {
  if (pageData.value.every((p) => selectedPkgnames.value.has(p.pkgname))) {
    selectedPkgnames.value = new Set();
  } else {
    selectedPkgnames.value = new Set(pageData.value.map((p) => p.pkgname));
  }
}

async function syncFromAur() {
  loading.value = true;
  try {
    await invoke("sync_from_aur");
    await pkgStore.fetchPackages();
    syncToolbar();
  } finally {
    loading.value = false;
  }
}

async function syncFromPkgbuild() {
  loading.value = true;
  try {
    await invoke("sync_from_pkgbuild");
    await pkgStore.fetchPackages();
    syncToolbar();
  } finally {
    loading.value = false;
  }
}

async function addSoftware() {
  const name = addPkgname.value.trim();
  if (!name) return;
  addLoading.value = true;
  try {
    await invoke("add_software", { pkgname: name });
    showAddDialog.value = false;
    addPkgname.value = "";
    await pkgStore.fetchPackages();
    syncToolbar();
  } finally {
    addLoading.value = false;
  }
}

function editSelected() {
  const arr = Array.from(selectedPkgnames.value);
  if (arr.length === 1) {
    router.push(`/packages/${arr[0]}`);
  }
}

async function updateAurInfo() {
  loading.value = true;
  try {
    const list = Array.from(selectedPkgnames.value);
    await invoke("update_aur_info", { pkgnameList: list.length ? list : null });
    await pkgStore.fetchPackages();
    syncToolbar();
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
      await pkgStore.fetchPackages();
      syncToolbar();
    }
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
    await pkgStore.fetchPackages();
    syncToolbar();
  } finally {
    loading.value = false;
  }
}

async function checkAll() {
  loading.value = true;
  footer.progress = { current: 0, total: 1 };
  try {
    await invoke("check_all_upstream");
    await pkgStore.fetchPackages();
    syncToolbar();
  } finally {
    loading.value = false;
    footer.progress = null;
  }
}
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="syncFromAur" :disabled="loading">从AUR同步</button>
      <button class="btn btn-primary" @click="syncFromPkgbuild" :disabled="loading">从PKGBUILD同步</button>
      <button class="btn btn-primary" @click="showAddDialog = true">添加</button>
      <button class="btn btn-outline" @click="editSelected" :disabled="selectedPkgnames.size !== 1">编辑</button>
      <button class="btn btn-outline" @click="updateAurInfo" :disabled="loading">更新AUR信息</button>
      <button class="btn btn-outline" @click="checkSelectedUpstream" :disabled="loading || selectedPkgnames.size === 0">更新上游信息</button>
      <button class="btn btn-danger" @click="deleteSelected" :disabled="selectedPkgnames.size === 0">删除</button>
      <button class="btn btn-outline" @click="checkAll" :disabled="loading">检查全部更新</button>
    </PageToolbar>

    <div class="card">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input type="checkbox" :checked="pageData.length > 0 && pageData.every(p => selectedPkgnames.has(p.pkgname))"
                :indeterminate="pageData.some(p => selectedPkgnames.has(p.pkgname)) && !pageData.every(p => selectedPkgnames.has(p.pkgname))"
                @change="toggleSelectAll" />
            </th>
            <th>包名</th>
            <th>类型</th>
            <th>检查器</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pkg in pageData" :key="pkg.pkgname"
            :class="{ 'row-selected': selectedPkgnames.has(pkg.pkgname) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedPkgnames.has(pkg.pkgname)"
                @change="toggleSelect(pkg.pkgname)" />
            </td>
            <td @click="router.push(`/packages/${pkg.pkgname}`)"><strong>{{ pkg.pkgname }}</strong></td>
            <td @click="router.push(`/packages/${pkg.pkgname}`)">{{ pkg.package_type_id }}</td>
            <td @click="router.push(`/packages/${pkg.pkgname}`)">{{ checkerText[pkg.checker_type_id] || pkg.checker_type_id }}</td>
            <td @click="router.push(`/packages/${pkg.pkgname}`)">
              <span class="status-badge" :class="pkg.is_outdated ? 'status-update_available' : 'status-up_to_date'">
                {{ pkg.is_outdated ? "需更新" : "已最新" }}
              </span>
            </td>
            <td>
              <button class="btn btn-outline" style="padding: 0.25rem 0.5rem; font-size: 0.75rem"
                @click.stop="pkgStore.checkVersion(pkg.pkgname)">
                检查
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <Teleport to="body">
      <div v-if="showAddDialog" class="modal-overlay" @click.self="showAddDialog = false">
        <div class="modal">
          <h3>添加软件包</h3>
          <input v-model="addPkgname" placeholder="输入包名" @keyup.enter="addSoftware" class="form-input" />
          <div class="modal-actions">
            <button class="btn btn-outline" @click="showAddDialog = false">取消</button>
            <button class="btn btn-primary" @click="addSoftware" :disabled="addLoading || !addPkgname.trim()">
              {{ addLoading ? "添加中..." : "确认" }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.pkg-table {
  width: 100%;
  border-collapse: collapse;
}
.pkg-table th {
  text-align: left;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
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
