<!--
  PackageDetail.vue - 软件包详情页面

  功能：
  - 显示软件包详细信息
  - 支持检查上游版本
  - 支持设置 License 和编程语言
  - 显示软件包状态

  路由参数：
  - pkgname: 软件包名称（从 URL 路径获取）

  数据来源：
  - get_software: 获取软件包信息
  - get_licenses: 获取 License 列表
  - get_languages: 获取编程语言列表
  - check_upstream_version: 检查上游版本
  - set_software_license: 设置 License
  - set_software_language: 设置编程语言
-->
<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareInfo, License, Language } from "../types";
import { FOOTER_KEY } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";

const route = useRoute();
const footer = inject(FOOTER_KEY)!;

/** 软件包信息 */
const pkg = ref<SoftwareInfo | null>(null);

/** 上游版本号 */
const upstreamVer = ref("");

/** 检查中状态 */
const checking = ref(false);

/** 错误信息 */
const error = ref("");

/** License 列表 */
const licenses = ref<License[]>([]);

/** 编程语言列表 */
const languages = ref<Language[]>([]);

/**
 * 组件挂载时加载数据
 */
onMounted(async () => {
  const pkgname = route.params.pkgname as string;
  try {
    pkg.value = await invoke<SoftwareInfo | null>("get_software", { pkgname });
    licenses.value = await invoke<License[]>("get_licenses");
    languages.value = await invoke<Language[]>("get_languages");
    if (pkg.value) {
      footer.infoText = `${pkg.value.pkgname}  |  ${pkg.value.is_outdated ? "需更新" : "已最新"}`;
    }
  } catch {
    error.value = "Failed to load package";
  }
});

/**
 * 检查上游版本
 * 调用后端检查此软件包的上游版本，更新上游版本号并标记为有更新
 */
async function checkUpdate() {
  if (!pkg.value) return;
  checking.value = true;
  error.value = "";
  try {
    upstreamVer.value = await invoke<string>("check_upstream_version", {
      pkgname: pkg.value.pkgname,
    });
    pkg.value.is_outdated = true;  // 检查到上游版本后标记为需更新
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
  }
}

/**
 * 设置软件包的 License
 * 调用后端更新软件包的许可证关联
 * @param licenseId - License ID，null 表示清除设置
 */
async function setLicense(licenseId: number | null) {
  if (!pkg.value || !pkg.value.software_id) return;
  try {
    await invoke("set_software_license", {
      softwareId: pkg.value.software_id,
      licenseId,
    });
    pkg.value.license_id = licenseId;  // 本地同步更新
  } catch (e) {
    error.value = String(e);
  }
}

/**
 * 设置软件包的编程语言
 * 调用后端更新软件包的编程语言关联
 * @param languageId - 编程语言 ID，null 表示清除设置
 */
async function setLanguage(languageId: number | null) {
  if (!pkg.value || !pkg.value.software_id) return;
  try {
    await invoke("set_software_language", {
      softwareId: pkg.value.software_id,
      languageId,
    });
    pkg.value.language_id = languageId;  // 本地同步更新
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="checkUpdate" :disabled="checking" v-if="pkg">
        {{ checking ? "检查中..." : "检查上游版本" }}
      </button>
    </PageToolbar>

    <!-- 错误提示区域 - 显示操作错误信息 -->
    <div v-if="error" class="card" style="border-color: var(--error); margin-bottom: 1rem">
      {{ error }}
    </div>

    <!-- 软件包信息卡片 - 显示包名、类型、上游地址、状态等详细信息 -->
    <div v-if="pkg" class="card">
      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">包名</td>
            <td>{{ pkg.pkgname }}</td>
          </tr>
          <tr>
            <td class="label">类型</td>
            <td>{{ pkg.package_type_id }}</td>
          </tr>
          <tr>
            <td class="label">上游地址</td>
            <td>{{ pkg.upstream_url || "-" }}</td>
          </tr>
          <tr>
            <td class="label">检查器类型</td>
            <td>{{ pkg.checker_type_id }}</td>
          </tr>
          <tr>
            <td class="label">状态</td>
            <td>
              <span class="status-badge" :class="pkg.is_outdated ? 'status-update_available' : 'status-up_to_date'">
                {{ pkg.is_outdated ? "需更新" : "已最新" }}
              </span>
            </td>
          </tr>
          <tr>
            <td class="label">License</td>
            <td>
              <select
                :value="pkg.license_id ?? ''"
                @change="setLicense(($event.target as HTMLSelectElement).value ? Number(($event.target as HTMLSelectElement).value) : null)"
                class="select-input"
              >
                <option value="">未设置</option>
                <option v-for="(lic, i) in licenses" :key="i" :value="lic.id">
                  {{ lic.spdx_id }} — {{ lic.full_name }}
                </option>
              </select>
            </td>
          </tr>
          <tr>
            <td class="label">编程语言</td>
            <td>
              <select
                :value="pkg.language_id ?? ''"
                @change="setLanguage(($event.target as HTMLSelectElement).value ? Number(($event.target as HTMLSelectElement).value) : null)"
                class="select-input"
              >
                <option value="">未设置</option>
                <option v-for="(lang, i) in languages" :key="i" :value="lang.id">
                  {{ lang.name }}
                </option>
              </select>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 加载中占位 -->
    <div v-else-if="!error" class="card">
      加载中...
    </div>
  </div>
</template>

<style scoped>
/* 信息表格样式 */
.info-table {
  width: 100%;
  border-collapse: collapse;
}

.info-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}

/* 标签列 - 灰色加粗，固定宽度 */
.info-table .label {
  color: var(--text-secondary);
  width: 120px;
  font-weight: 600;
}

/* 下拉选择框样式 */
.select-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8125rem;
  min-width: 240px;
}
</style>
