<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, ArrowRight } from "@lucide/vue";
import { FOOTER_KEY } from "../composables/footer";
import { useSoftwareForm } from "../composables/useSoftwareForm";
import SoftwareFormModal from "../components/SoftwareFormModal.vue";
import SoftwareInfoCard from "../components/SoftwareInfoCard.vue";
import SoftwareAurCard from "../components/SoftwareAurCard.vue";
import SoftwareUpstreamCard from "../components/SoftwareUpstreamCard.vue";
import SoftwareToolbar from "../components/SoftwareToolbar.vue";

const route = useRoute();
const router = useRouter();
const footer = inject(FOOTER_KEY)!;

const { saving, error, detail, save, init } = useSoftwareForm();

const showEditModal = ref(false);
const deleting = ref(false);
const updatingAur = ref(false);
const updatingPkgbuild = ref(false);
const checking = ref(false);
const successMsg = ref("");
const prevPkgname = ref<string | null>(null);
const nextPkgname = ref<string | null>(null);

function syncFooter() {
  if (detail.value) {
    footer.infoText = `${detail.value.pkgname}  |  ${detail.value.is_outdated ? "需更新" : "已最新"}`;
  }
}

async function loadNav(pkgname: string) {
  try {
    const [prev, next] = await invoke<[string | null, string | null]>("get_prev_next_software", { pkgname });
    prevPkgname.value = prev;
    nextPkgname.value = next;
  } catch {
    // ignore
  }
}

onMounted(async () => {
  const pkgname = route.params.pkgname as string;
  await init("edit", pkgname);
  await loadNav(pkgname);
  syncFooter();
});

async function checkUpdate() {
  if (!detail.value) return;
  checking.value = true;
  error.value = "";
  try {
    await invoke<string>("check_upstream_version", { pkgname: detail.value.pkgname });
    await init("edit", detail.value.pkgname);
    syncFooter();
    successMsg.value = "上游版本检查完成";
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

async function updateAurInfo() {
  if (!detail.value) return;
  updatingAur.value = true;
  error.value = "";
  try {
    await invoke<number>("update_aur_info", { pkgnameList: [detail.value.pkgname] });
    await init("edit", detail.value.pkgname);
    successMsg.value = "AUR 信息更新完成";
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingAur.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

async function updatePkgbuild() {
  if (!detail.value) return;
  updatingPkgbuild.value = true;
  error.value = "";
  try {
    await invoke<number>("sync_from_pkgbuild", { pkgname: detail.value.pkgname });
    await init("edit", detail.value.pkgname);
    successMsg.value = "PKGBUILD 信息更新完成";
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingPkgbuild.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

async function handleDelete() {
  if (!detail.value?.software_id) return;
  if (!confirm(`确定要删除软件包 "${detail.value.pkgname}" 吗？`)) return;

  deleting.value = true;
  error.value = "";
  try {
    await invoke("delete_software", { softwareId: detail.value.software_id });
    router.push("/packages");
  } catch (e) {
    error.value = String(e);
  } finally {
    deleting.value = false;
  }
}

async function handleEditSave() {
  await save("edit");
  if (!error.value && detail.value) {
    await init("edit", detail.value.pkgname);
    syncFooter();
  }
}

function navigate(direction: "prev" | "next") {
  const pkgname = direction === "prev" ? prevPkgname.value : nextPkgname.value;
  if (pkgname) {
    router.push(`/packages/${pkgname}`);
  }
}
</script>

<template>
  <div class="detail-header">
    <div class="header-left">
      <button class="btn btn-outline" @click="router.push('/packages')" title="返回列表">
        ← 返回
      </button>
    </div>
    <div class="header-center">
      <h1 class="pkg-title">{{ detail?.pkgname || "加载中..." }}</h1>
    </div>
    <div class="header-right">
      <button class="nav-btn" :class="{ disabled: !prevPkgname }" @click="navigate('prev')" title="上一个">
        <ArrowLeft :size="20" />
        <span>{{ prevPkgname || "" }}</span>
      </button>
      <button class="nav-btn" :class="{ disabled: !nextPkgname }" @click="navigate('next')" title="下一个">
        <span>{{ nextPkgname || "" }}</span>
        <ArrowRight :size="20" />
      </button>
    </div>
  </div>

  <div class="detail-content">
    <div v-if="error" class="card msg-error">{{ error }}</div>
    <div v-if="successMsg" class="card msg-success">{{ successMsg }}</div>

    <div v-if="detail" class="detail-cards">
      <SoftwareInfoCard :detail="detail" />
      <SoftwareAurCard
        :aur-version="detail.aur_version"
        :aur-pkgdesc="detail.aur_pkgdesc"
        :aur-last-updated="detail.aur_last_updated"
      />
      <SoftwareUpstreamCard
        :upstream-version="detail.upstream_version"
        :upstream-last-checked="detail.upstream_last_checked"
      />
    </div>

    <div v-else-if="!error" class="card empty-card">加载中...</div>
  </div>

  <div class="detail-footer">
    <SoftwareToolbar
      :deleting="deleting"
      :updating-aur="updatingAur"
      :updating-pkgbuild="updatingPkgbuild"
      :checking="checking"
      :disabled="saving"
      @edit="showEditModal = true"
      @delete="handleDelete"
      @update-aur="updateAurInfo"
      @update-pkgbuild="updatePkgbuild"
      @check-update="checkUpdate"
    />
  </div>

  <SoftwareFormModal
    :show="showEditModal"
    mode="edit"
    :pkgname="detail?.pkgname"
    @close="showEditModal = false"
    @saved="handleEditSave"
  />
</template>

<style scoped>
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
}

.header-left, .header-right { display: flex; align-items: center; gap: 0.5rem; }
.header-center { flex: 1; text-align: center; }

.pkg-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.nav-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
}

.nav-btn:hover:not(.disabled) {
  border-color: var(--accent);
  color: var(--accent);
  background-color: var(--bg-hover);
}

.nav-btn.disabled { opacity: 0.4; cursor: not-allowed; }

.detail-content {
  padding: 1.25rem;
  max-width: 900px;
  margin: 0 auto;
}

.detail-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1rem;
  margin-top: 0.5rem;
}

.msg-error { border-color: var(--error); color: var(--error); padding: 0.75rem; }
.msg-success { border-color: var(--success); color: var(--success); padding: 0.75rem; }
.empty-card { padding: 2rem; text-align: center; color: var(--text-secondary); }

.detail-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.75rem 1.25rem;
  background-color: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.spinning { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
