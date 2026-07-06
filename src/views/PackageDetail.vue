<!--
  PackageDetail.vue - 软件包详情页面（重新设计）

  页面布局：
  1. 顶部导航栏：包名 + 返回列表 + 上一个/下一个导航
  2. 主体区域：三个卡片 - 基本信息、AUR信息、上游信息
  3. 底部工具栏：编辑、删除、更新AUR、更新PKGBUILD、检查上游版本

  路由参数：
  - pkgname: 软件包名称（从 URL 路径获取）
-->
<script setup lang="ts">
import { ref, onMounted, inject, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, ArrowRight, Trash2, RefreshCw, FileCode, GitBranch, Edit } from "@lucide/vue";
import { FOOTER_KEY } from "../composables/footer";
import { useSoftwareForm, pkgTypes, checkerTypes } from "../composables/useSoftwareForm";
import SoftwareFormModal from "../components/SoftwareFormModal.vue";

const route = useRoute();
const router = useRouter();
const footer = inject(FOOTER_KEY)!;

const {
  saving, error, licenses, languages, detail,
  save, init,
} = useSoftwareForm();

const showEditModal = ref(false);
const deleting = ref(false);
const updatingAur = ref(false);
const updatingPkgbuild = ref(false);
const checking = ref(false);
const successMsg = ref("");
const prevPkgname = ref<string | null>(null);
const nextPkgname = ref<string | null>(null);

const pkgTypeName = computed(() => {
  return pkgTypes.find(t => t.id === detail.value?.package_type_id)?.label || '未知';
});

const checkerTypeName = computed(() => {
  return checkerTypes.find(c => c.id === detail.value?.checker_type_id)?.label || '未知';
});

const licenseName = computed(() => {
  return licenses.value.find(l => l.id === detail.value?.license_id)?.spdx_id || '未设置';
});

const languageName = computed(() => {
  return languages.value.find(l => l.id === detail.value?.language_id)?.name || '未设置';
});

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

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  const date = new Date(ts * 1000);
  return date.toLocaleString("zh-CN");
}
</script>

<template>
  <!-- 顶部导航栏 -->
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
      <button
        class="nav-btn"
        :class="{ disabled: !prevPkgname }"
        @click="navigate('prev')"
        title="上一个"
      >
        <ArrowLeft :size="20" />
        <span>{{ prevPkgname || "" }}</span>
      </button>
      <button
        class="nav-btn"
        :class="{ disabled: !nextPkgname }"
        @click="navigate('next')"
        title="下一个"
      >
        <span>{{ nextPkgname || "" }}</span>
        <ArrowRight :size="20" />
      </button>
    </div>
  </div>

  <!-- 消息提示 -->
  <div class="detail-content">
    <div v-if="error" class="card msg-error">{{ error }}</div>
    <div v-if="successMsg" class="card msg-success">{{ successMsg }}</div>

    <!-- 内容区域 -->
    <div v-if="detail" class="detail-cards">
      <!-- 基本信息卡片 -->
      <div class="info-card">
        <div class="card-header">
          <h3 class="card-title">基本信息</h3>
          <span :class="['status-badge', detail.is_outdated ? 'outdated' : 'latest']">
            {{ detail.is_outdated ? '需更新' : '已最新' }}
          </span>
        </div>
        <table class="info-table">
          <tbody>
            <tr>
              <td class="label">包名</td>
              <td class="value">{{ detail.pkgname }}</td>
            </tr>
            <tr>
              <td class="label">软件类型</td>
              <td class="value">{{ pkgTypeName }}</td>
            </tr>
            <tr>
              <td class="label">检查器类型</td>
              <td class="value">{{ checkerTypeName }}</td>
            </tr>
            <tr>
              <td class="label">上游地址</td>
              <td class="value url-value">
                <a v-if="detail.upstream_url" :href="detail.upstream_url" target="_blank">
                  {{ detail.upstream_url }}
                </a>
                <span v-else>未设置</span>
              </td>
            </tr>
            <tr>
              <td class="label">License</td>
              <td class="value">{{ licenseName }}</td>
            </tr>
            <tr>
              <td class="label">编程语言</td>
              <td class="value">{{ languageName }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- AUR 信息卡片 -->
      <div class="info-card">
        <div class="card-header">
          <h3 class="card-title">AUR 信息</h3>
          <span class="card-subtitle">来自 AUR 仓库</span>
        </div>
        <table class="info-table">
          <tbody>
            <tr>
              <td class="label">AUR 版本</td>
              <td class="value">{{ detail.aur_version || '—' }}</td>
            </tr>
            <tr>
              <td class="label">包描述</td>
              <td class="value">{{ detail.aur_pkgdesc || '—' }}</td>
            </tr>
            <tr>
              <td class="label">更新时间</td>
              <td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 上游信息卡片 -->
      <div class="info-card">
        <div class="card-header">
          <h3 class="card-title">上游版本信息</h3>
          <span class="card-subtitle">最新版本检查结果</span>
        </div>
        <table class="info-table">
          <tbody>
            <tr>
              <td class="label">上游版本</td>
              <td class="value">{{ detail.upstream_version || '—' }}</td>
            </tr>
            <tr>
              <td class="label">上次检查</td>
              <td class="value">{{ detail.upstream_last_checked || '—' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-else-if="!error" class="card empty-card">
      加载中...
    </div>
  </div>

  <!-- 底部工具栏 -->
  <div class="detail-footer">
    <div class="footer-left">
      <button class="footer-btn" @click="showEditModal = true" :disabled="saving">
        <Edit :size="16" />
        <span>编辑</span>
      </button>
      <button class="footer-btn danger" @click="handleDelete" :disabled="deleting || saving">
        <Trash2 :size="16" />
        <span>删除</span>
      </button>
    </div>
    <div class="footer-right">
      <button class="footer-btn" @click="updateAurInfo" :disabled="updatingAur || saving">
        <GitBranch :size="16" />
        <span>{{ updatingAur ? '更新中...' : '更新 AUR 信息' }}</span>
      </button>
      <button class="footer-btn" @click="updatePkgbuild" :disabled="updatingPkgbuild || saving">
        <FileCode :size="16" />
        <span>{{ updatingPkgbuild ? '更新中...' : '更新 PKGBUILD' }}</span>
      </button>
      <button class="footer-btn primary" @click="checkUpdate" :disabled="checking || saving">
        <RefreshCw :size="16" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查上游版本' }}</span>
      </button>
    </div>
  </div>

  <!-- 编辑弹窗 -->
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

.header-left {
  display: flex;
  align-items: center;
}

.header-center {
  flex: 1;
  text-align: center;
}

.pkg-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
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

.nav-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

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

.info-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.card-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.card-subtitle {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.status-badge {
  font-size: 0.75rem;
  font-weight: 500;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
}

.status-badge.latest {
  background-color: var(--success-bg);
  color: var(--success);
}

.status-badge.outdated {
  background-color: var(--warning-bg);
  color: var(--warning);
}

.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 0.5rem 0.25rem;
  font-size: 0.8125rem;
}

.info-table .label {
  width: 80px;
  color: var(--text-secondary);
  vertical-align: top;
}

.info-table .value {
  color: var(--text-primary);
}

.url-value a {
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}

.url-value a:hover {
  text-decoration: underline;
}

.msg-error {
  border-color: var(--error);
  color: var(--error);
  padding: 0.75rem;
}

.msg-success {
  border-color: var(--success);
  color: var(--success);
  padding: 0.75rem;
}

.empty-card {
  padding: 2rem;
  text-align: center;
  color: var(--text-secondary);
}

.detail-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.25rem;
  background-color: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.footer-left,
.footer-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.8125rem;
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
