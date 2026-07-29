<!--
  SoftwareDetailModal.vue - 软件详情弹窗组件

  功能：
  - 显示软件包的完整信息（基本信息、AUR 信息、上游信息）
  - 支持前后导航（上一个/下一个软件包）
  - 提供操作按钮：编辑、删除、更新 AUR、同步 PKGBUILD、检查更新

  使用组件：
  - SoftwareInfoTable: 信息表格组件
  - SoftwareStatusRow: 状态行组件
  - SoftwareSideCards: 侧边信息卡片组件
  - SoftwareInfoCard: 基本信息卡片
  - SoftwareAurCard: AUR 信息卡片
  - SoftwareUpstreamCard: 上游信息卡片
-->
<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareDetail, Language } from "../types";
import Modal from "./common/Modal.vue";
import SoftwareFormModal from "./SoftwareFormModal.vue";
import FloatingNav from "./FloatingNav.vue";
import DetailToolbar from "./DetailToolbar.vue";
import SoftwareInfoCard from "./SoftwareInfoCard.vue";
import SoftwareInfoTable from "./SoftwareInfoTable.vue";
import SoftwareStatusRow from "./SoftwareStatusRow.vue";
import SoftwareSideCards from "./SoftwareSideCards.vue";

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
const languages = ref<Language[]>([]);

async function loadLanguages() {
  try {
    languages.value = await invoke<Language[]>("get_languages");
  } catch {
    // ignore
  }
}

onMounted(() => {
  loadLanguages();
});

async function loadSoftware() {
  if (!props.pkgname) return;
  loading.value = true;
  error.value = "";
  try {
    detail.value = await invoke<SoftwareDetail | null>("get_software_detail", {
      pkgname: props.pkgname,
    });
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
    const [prev, next] = await invoke<[string | null, string | null]>(
      "get_prev_next_software",
      { pkgname: props.pkgname }
    );
    prevPkgname.value = prev;
    nextPkgname.value = next;
  } catch {
    /* ignore */
  }
}

function navigate(direction: "prev" | "next") {
  const target =
    direction === "prev" ? prevPkgname.value : nextPkgname.value;
  if (target) emit("navigate", target);
}

async function updateAurInfo() {
  if (!detail.value) return;
  updatingAur.value = true;
  error.value = "";
  try {
    await invoke<number>("update_aur_info", {
      pkgnameList: [detail.value.pkgname],
    });
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
    await invoke<number>("sync_from_pkgbuild", {
      pkgname: detail.value.pkgname,
    });
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
    await invoke<string>("check_upstream_version", {
      pkgname: detail.value.pkgname,
    });
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

watch(
  () => props.pkgname,
  () => {
    if (props.show) loadSoftware();
  }
);
watch(
  () => props.show,
  (val) => {
    if (!val) showEditModal.value = false;
  }
);
</script>

<template>
  <Modal :show="show" width="720px" hide-header @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>

    <div class="detail-header">
      <h3 class="pkg-title">{{ detail?.pkgname || "软件详情" }}</h3>
    </div>

    <FloatingNav :prev="prevPkgname" :next="nextPkgname" @navigate="navigate" />

    <div v-if="loading" class="loading-text">加载中...</div>

    <div v-else-if="detail" class="detail-content">
      <div class="section">
        <div class="badge-row">
          <span
            :class="[
              'status-badge',
              detail.is_outdated ? 'outdated' : 'latest',
            ]"
          >
            {{ detail.is_outdated ? "需更新" : "已最新" }}
          </span>
        </div>

        <SoftwareInfoTable :detail="detail" :languages="languages" />
        <SoftwareStatusRow :detail="detail" />
        <SoftwareInfoCard :detail="detail" />
      </div>

      <SoftwareSideCards :detail="detail" />
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
.detail-header {
  text-align: center;
  margin-bottom: 0.75rem;
}
.pkg-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.loading-text {
  color: var(--text-secondary);
  text-align: center;
  padding: 1.5rem 0;
}
.detail-content {
  max-height: 75vh;
  overflow: visible;
}

.badge-row {
  display: flex;
  justify-content: center;
  margin-bottom: 0.75rem;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-badge.outdated {
  background-color: var(--warning-bg);
  color: var(--warning);
}

.status-badge.latest {
  background-color: var(--success-bg);
  color: var(--success);
}

.section {
  margin-bottom: 1rem;
}
</style>