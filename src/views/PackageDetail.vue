<!--
  PackageDetail.vue - 软件包详情/编辑页面

  功能：
  - 显示并编辑软件包详细信息
  - 支持检查上游版本
  - 支持设置 License 和编程语言
  - 支持保存所有可编辑字段

  路由参数：
  - pkgname: 软件包名称（从 URL 路径获取）
-->
<script setup lang="ts">
import { ref, onMounted, inject, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareInfo, License, Language } from "../types";
import { FOOTER_KEY } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";

const route = useRoute();
const router = useRouter();
const footer = inject(FOOTER_KEY)!;

const pkg = ref<SoftwareInfo | null>(null);
const checking = ref(false);
const saving = ref(false);
const error = ref("");
const successMsg = ref("");
const licenses = ref<License[]>([]);
const languages = ref<Language[]>([]);

// 编辑表单字段（从原始数据初始化）
const form = ref({
  upstream_url: "",
  package_type_id: 1,
  checker_type_id: 7,
  is_outdated: false,
  check_test_versions: false,
  check_binary_files: false,
  auto_check_enabled: true,
  license_id: null as number | null,
  language_id: null as number | null,
});

const isDirty = computed(() => {
  if (!pkg.value) return false;
  return (
    form.value.upstream_url !== (pkg.value.upstream_url ?? "") ||
    form.value.package_type_id !== pkg.value.package_type_id ||
    form.value.checker_type_id !== pkg.value.checker_type_id ||
    form.value.is_outdated !== pkg.value.is_outdated ||
    form.value.check_test_versions !== pkg.value.check_test_versions ||
    form.value.check_binary_files !== pkg.value.check_binary_files ||
    form.value.auto_check_enabled !== pkg.value.auto_check_enabled ||
    form.value.license_id !== pkg.value.license_id ||
    form.value.language_id !== pkg.value.language_id
  );
});

function syncFormFromPkg() {
  if (!pkg.value) return;
  form.value = {
    upstream_url: pkg.value.upstream_url ?? "",
    package_type_id: pkg.value.package_type_id,
    checker_type_id: pkg.value.checker_type_id,
    is_outdated: pkg.value.is_outdated,
    check_test_versions: pkg.value.check_test_versions,
    check_binary_files: pkg.value.check_binary_files,
    auto_check_enabled: pkg.value.auto_check_enabled,
    license_id: pkg.value.license_id,
    language_id: pkg.value.language_id,
  };
}

onMounted(async () => {
  const pkgname = route.params.pkgname as string;
  try {
    pkg.value = await invoke<SoftwareInfo | null>("get_software", { pkgname });
    licenses.value = await invoke<License[]>("get_licenses");
    languages.value = await invoke<Language[]>("get_languages");
    syncFormFromPkg();
    if (pkg.value) {
      footer.infoText = `${pkg.value.pkgname}  |  ${pkg.value.is_outdated ? "需更新" : "已最新"}`;
    }
  } catch {
    error.value = "加载软件包信息失败";
  }
});

async function checkUpdate() {
  if (!pkg.value) return;
  checking.value = true;
  error.value = "";
  try {
    await invoke<string>("check_upstream_version", { pkgname: pkg.value.pkgname });
    const updated = await invoke<SoftwareInfo | null>("get_software", { pkgname: pkg.value.pkgname });
    if (updated) {
      pkg.value = updated;
      syncFormFromPkg();
      footer.infoText = `${pkg.value.pkgname}  |  ${pkg.value.is_outdated ? "需更新" : "已最新"}`;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
  }
}

async function save() {
  if (!pkg.value || !pkg.value.software_id) return;
  saving.value = true;
  error.value = "";
  successMsg.value = "";
  try {
    await invoke("update_software", {
      softwareId: pkg.value.software_id,
      pkgname: pkg.value.pkgname,
      upstreamUrl: form.value.upstream_url || null,
      packageType: form.value.package_type_id,
      checkerType: form.value.checker_type_id,
      isOutdated: form.value.is_outdated,
      checkTestVersions: form.value.check_test_versions,
      checkBinaryFiles: form.value.check_binary_files,
      autoCheckEnabled: form.value.auto_check_enabled,
      licenseId: form.value.license_id,
      languageId: form.value.language_id,
    });
    // 刷新数据
    const updated = await invoke<SoftwareInfo | null>("get_software", { pkgname: pkg.value.pkgname });
    if (updated) {
      pkg.value = updated;
      syncFormFromPkg();
      footer.infoText = `${pkg.value.pkgname}  |  ${pkg.value.is_outdated ? "需更新" : "已最新"}`;
    }
    successMsg.value = "保存成功";
    setTimeout(() => { successMsg.value = ""; }, 2000);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

function reset() {
  syncFormFromPkg();
  error.value = "";
  successMsg.value = "";
}

const pkgTypes = [
  { id: 1, label: "编译安装" },
  { id: 2, label: "二进制包" },
  { id: 3, label: "Git 仓库" },
  { id: 4, label: "AppImage" },
];

const checkerTypes = [
  { id: 1, label: "GitHub Release" },
  { id: 2, label: "GitHub Tag" },
  { id: 3, label: "Gitee" },
  { id: 4, label: "GitLab" },
  { id: 5, label: "重定向" },
  { id: 6, label: "HTTP 页面解析" },
  { id: 7, label: "手动" },
];
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-outline" @click="router.push('/packages')" :disabled="saving">
        ← 返回列表
      </button>
      <div style="flex: 1" />
      <button class="btn btn-primary" @click="checkUpdate" :disabled="checking || saving">
        {{ checking ? "检查中..." : "检查上游版本" }}
      </button>
      <button class="btn btn-primary" @click="save" :disabled="saving || !isDirty">
        {{ saving ? "保存中..." : "保存" }}
      </button>
      <button class="btn btn-outline" @click="reset" :disabled="saving || !isDirty">
        还原
      </button>
    </PageToolbar>

    <div v-if="error" class="card msg-error" style="margin-top: 1rem">
      {{ error }}
    </div>
    <div v-if="successMsg" class="card msg-success" style="margin-top: 1rem">
      {{ successMsg }}
    </div>

    <div v-if="pkg" class="card" style="margin-top: 1rem">
      <h3 class="section-title">{{ pkg.pkgname }}</h3>

      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">包名</td>
            <td class="value">{{ pkg.pkgname }}</td>
          </tr>
          <tr>
            <td class="label">上游地址</td>
            <td>
              <input
                v-model="form.upstream_url"
                class="form-input"
                placeholder="https://..."
              />
            </td>
          </tr>
          <tr>
            <td class="label">软件类型</td>
            <td>
              <select v-model.number="form.package_type_id" class="form-select">
                <option v-for="t in pkgTypes" :key="t.id" :value="t.id">{{ t.label }}</option>
              </select>
            </td>
          </tr>
          <tr>
            <td class="label">检查器类型</td>
            <td>
              <select v-model.number="form.checker_type_id" class="form-select">
                <option v-for="c in checkerTypes" :key="c.id" :value="c.id">{{ c.label }}</option>
              </select>
            </td>
          </tr>
          <tr>
            <td class="label">状态</td>
            <td>
              <label class="checkbox-label">
                <input type="checkbox" v-model="form.is_outdated" />
                <span>需更新</span>
              </label>
            </td>
          </tr>
          <tr>
            <td class="label">License</td>
            <td>
              <select v-model="form.license_id" class="form-select">
                <option :value="null">未设置</option>
                <option v-for="(lic, i) in licenses" :key="i" :value="lic.id">
                  {{ lic.spdx_id }} — {{ lic.full_name }}
                </option>
              </select>
            </td>
          </tr>
          <tr>
            <td class="label">编程语言</td>
            <td>
              <select v-model="form.language_id" class="form-select">
                <option :value="null">未设置</option>
                <option v-for="(lang, i) in languages" :key="i" :value="lang.id">
                  {{ lang.name }}
                </option>
              </select>
            </td>
          </tr>
        </tbody>
      </table>

      <h3 class="section-title" style="margin-top: 1.5rem">检查选项</h3>
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">自动检查更新</td>
            <td>
              <label class="checkbox-label">
                <input type="checkbox" v-model="form.auto_check_enabled" />
                <span>启用</span>
              </label>
            </td>
          </tr>
          <tr>
            <td class="label">检查测试版本</td>
            <td>
              <label class="checkbox-label">
                <input type="checkbox" v-model="form.check_test_versions" />
                <span>启用</span>
              </label>
            </td>
          </tr>
          <tr>
            <td class="label">检查二进制文件</td>
            <td>
              <label class="checkbox-label">
                <input type="checkbox" v-model="form.check_binary_files" />
                <span>启用</span>
              </label>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else-if="!error" class="card" style="margin-top: 1rem">
      加载中...
    </div>
  </div>
</template>

<style scoped>
.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.75rem;
}

.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 0.625rem 0.75rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}

.info-table .label {
  color: var(--text-secondary);
  width: 140px;
  font-weight: 600;
  white-space: nowrap;
}

.info-table .value {
  color: var(--text-primary);
}

.form-input,
.form-select {
  padding: 0.375rem 0.625rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8125rem;
  width: 100%;
  max-width: 400px;
}

.form-select {
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239a9cb8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.5rem center;
  padding-right: 2rem;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--accent);
}

.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}

.msg-error {
  border-color: var(--error);
  color: var(--error);
}

.msg-success {
  border-color: var(--success);
  color: var(--success);
}
</style>
