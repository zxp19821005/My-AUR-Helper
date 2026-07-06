<!--
  PackageDetail.vue - 软件包详情/编辑页面

  路由参数：
  - pkgname: 软件包名称（从 URL 路径获取）
-->
<script setup lang="ts">
import { ref, onMounted, inject, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY } from "../composables/footer";
import { useSoftwareForm, pkgTypes, checkerTypes } from "../composables/useSoftwareForm";
import PageToolbar from "../components/PageToolbar.vue";

const route = useRoute();
const router = useRouter();
const footer = inject(FOOTER_KEY)!;

const {
  saving, error, form, licenses, languages, detail,
  isDirty, save, resetForm, init,
} = useSoftwareForm();

const checking = ref(false);
const successMsg = ref("");

const dirty = computed(() => isDirty(detail.value));

function syncFooter() {
  if (detail.value) {
    footer.infoText = `${detail.value.pkgname}  |  ${detail.value.is_outdated ? "需更新" : "已最新"}`;
  }
}

onMounted(async () => {
  const pkgname = route.params.pkgname as string;
  await init("edit", pkgname);
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
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
  }
}

async function handleSave() {
  const ok = await save("edit");
  if (ok) {
    if (detail.value) {
      await init("edit", detail.value.pkgname);
      syncFooter();
    }
    successMsg.value = "保存成功";
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

function handleReset() {
  resetForm("edit");
  successMsg.value = "";
}
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
      <button class="btn btn-primary" @click="handleSave" :disabled="saving || !dirty">
        {{ saving ? "保存中..." : "保存" }}
      </button>
      <button class="btn btn-outline" @click="handleReset" :disabled="saving || !dirty">
        还原
      </button>
    </PageToolbar>

    <div v-if="error" class="card msg-error" style="margin-top: 1rem">
      {{ error }}
    </div>
    <div v-if="successMsg" class="card msg-success" style="margin-top: 1rem">
      {{ successMsg }}
    </div>

    <div v-if="detail" class="card" style="margin-top: 1rem">
      <h3 class="section-title">{{ detail.pkgname }}</h3>

      <table class="info-table">
        <tbody>
          <tr>
            <td class="label">包名</td>
            <td class="value">{{ detail.pkgname }}</td>
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

.msg-error {
  border-color: var(--error);
  color: var(--error);
}

.msg-success {
  border-color: var(--success);
  color: var(--success);
}
</style>
