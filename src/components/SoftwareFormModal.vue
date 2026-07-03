<!--
  SoftwareFormModal.vue - 软件包添加/编辑弹窗

  Props:
  - show: boolean - 是否显示弹窗
  - mode: 'add' | 'edit' - 模式
  - pkgname: string - 编辑时的包名

  Events:
  - close: 关闭弹窗
  - saved: 保存成功
-->
<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareInfo, License, Language } from "../types";
import { X } from "@lucide/vue";

const props = defineProps<{
  show: boolean;
  mode: "add" | "edit";
  pkgname?: string;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const saving = ref(false);
const error = ref("");
const pkg = ref<SoftwareInfo | null>(null);
const licenses = ref<License[]>([]);
const languages = ref<Language[]>([]);

const form = ref({
  pkgname: "",
  upstream_url: "",
  package_type_id: 1,
  checker_type_id: 1,  // 默认 GitHub Release
  is_outdated: false,
  check_test_versions: false,
  check_binary_files: false,
  auto_check_enabled: false,  // 默认关闭
  license_id: null as number | null,
  language_id: null as number | null,
});

const isDirty = computed(() => {
  if (props.mode === "add") return true;
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

const canSave = computed(() => {
  if (props.mode === "add") return form.value.pkgname.trim().length > 0;
  return isDirty.value;
});

async function loadEnums() {
  try {
    licenses.value = await invoke<License[]>("get_licenses");
    languages.value = await invoke<Language[]>("get_languages");
  } catch {
    // ignore
  }
}

async function loadSoftware() {
  if (props.mode !== "edit" || !props.pkgname) return;
  try {
    const data = await invoke<SoftwareInfo | null>("get_software", { pkgname: props.pkgname });
    pkg.value = data;
    if (data) {
      form.value = {
        pkgname: data.pkgname,
        upstream_url: data.upstream_url ?? "",
        package_type_id: data.package_type_id,
        checker_type_id: data.checker_type_id,
        is_outdated: data.is_outdated,
        check_test_versions: data.check_test_versions,
        check_binary_files: data.check_binary_files,
        auto_check_enabled: data.auto_check_enabled,
        license_id: data.license_id,
        language_id: data.language_id,
      };
    }
  } catch (e) {
    error.value = String(e);
  }
}

function resetForm() {
  if (props.mode === "edit" && pkg.value) {
    form.value = {
      pkgname: pkg.value.pkgname,
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
  } else {
    form.value = {
      pkgname: "",
      upstream_url: "",
      package_type_id: 1,
      checker_type_id: 1,  // 默认 GitHub Release
      is_outdated: false,
      check_test_versions: false,
      check_binary_files: false,
      auto_check_enabled: false,
      license_id: null,
      language_id: null,
    };
  }
  error.value = "";
}

/** 根据包名自动检测软件类型和其他选项 */
function autoDetectFromPkgname() {
  const name = form.value.pkgname.trim().toLowerCase();
  if (!name) return;

  // 根据包名后缀判断软件类型
  if (name.endsWith("-bin")) {
    form.value.package_type_id = 2;  // Binary
    form.value.check_binary_files = true;
  } else if (name.endsWith("-git")) {
    form.value.package_type_id = 3;  // Git
  } else if (name.endsWith("-appimage")) {
    form.value.package_type_id = 4;  // AppImage
  } else {
    form.value.package_type_id = 1;  // Compiled
    form.value.check_binary_files = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      error.value = "";
      loadEnums();
      resetForm();
      if (props.mode === "edit") loadSoftware();
    }
  }
);

// 添加模式下，监听包名变化自动检测设置
watch(
  () => form.value.pkgname,
  () => {
    if (props.mode === "add") {
      autoDetectFromPkgname();
    }
  }
);

async function save() {
  saving.value = true;
  error.value = "";
  try {
    if (props.mode === "add") {
      await invoke("add_software", {
        pkgname: form.value.pkgname.trim(),
        upstreamUrl: form.value.upstream_url || null,
        packageType: form.value.package_type_id,
        checkerType: form.value.checker_type_id,
        checkTestVersions: form.value.check_test_versions,
        checkBinaryFiles: form.value.check_binary_files,
        autoCheckEnabled: form.value.auto_check_enabled,
        licenseId: form.value.license_id,
        languageId: form.value.language_id,
      });
    } else {
      if (!pkg.value?.software_id) return;
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
    }
    emit("saved");
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
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
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal">
        <div class="modal-header">
          <h3>{{ mode === "add" ? "添加软件包" : "编辑软件包" }}</h3>
          <button class="modal-close" @click="emit('close')">
            <X :size="18" />
          </button>
        </div>

        <div v-if="error" class="modal-error">{{ error }}</div>

        <div class="modal-body">
          <table class="form-table">
            <tbody>
              <tr v-if="mode === 'add'">
                <td class="label">包名 *</td>
                <td>
                  <input v-model="form.pkgname" class="form-input" placeholder="输入包名" />
                </td>
              </tr>
              <tr v-else>
                <td class="label">包名</td>
                <td class="value">{{ form.pkgname }}</td>
              </tr>
              <tr>
                <td class="label">上游地址</td>
                <td>
                  <input v-model="form.upstream_url" class="form-input" placeholder="https://..." />
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
              <tr v-if="mode === 'edit'">
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

        <div class="modal-footer">
          <button class="btn btn-outline" @click="emit('close')">取消</button>
          <button class="btn btn-primary" @click="save" :disabled="saving || !canSave">
            {{ saving ? "保存中..." : "确认" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
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
  background: var(--bg-card);
  border-radius: 12px;
  min-width: 480px;
  max-width: 560px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}
.modal-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}
.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
}
.modal-close:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}
.modal-error {
  padding: 0.5rem 1.25rem;
  color: var(--error);
  font-size: 0.8125rem;
  background: rgba(239, 83, 80, 0.08);
}
.modal-body {
  padding: 1rem 1.25rem;
  overflow-y: auto;
  flex: 1;
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--border);
}

.form-table {
  width: 100%;
  border-collapse: collapse;
}
.form-table td {
  padding: 0.5rem 0;
  font-size: 0.875rem;
  vertical-align: middle;
}
.form-table .label {
  color: var(--text-secondary);
  font-weight: 600;
  width: 110px;
  white-space: nowrap;
}
.form-table .value {
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
  max-width: 360px;
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
</style>
