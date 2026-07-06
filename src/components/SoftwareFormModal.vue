<!--
  SoftwareFormModal.vue - 软件包添加/编辑弹窗（两栏布局）

  Props:
  - show: boolean - 是否显示弹窗
  - mode: 'add' | 'edit' - 模式
  - pkgname: string - 编辑时的包名

  Events:
  - close: 关闭弹窗
  - saved: 保存成功
-->
<script setup lang="ts">
import { watch, computed } from "vue";
import { useSoftwareForm, pkgTypes, checkerTypes } from "../composables/useSoftwareForm";
import Modal from "./common/Modal.vue";

const props = defineProps<{
  show: boolean;
  mode: "add" | "edit";
  pkgname?: string;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const {
  saving, error, form, detail, licenses, languages,
  isDirty, canSave, save, init, autoDetectFromPkgname,
} = useSoftwareForm();

const dirty = computed(() => isDirty(detail.value));
const canSaveBtn = computed(() => canSave(props.mode, dirty.value));

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  const date = new Date(ts * 1000);
  return date.toLocaleString("zh-CN");
}

function formatDateTime(dt: string | null): string {
  if (!dt) return "—";
  return dt;
}

watch(
  () => props.show,
  (val) => {
    if (val) init(props.mode, props.pkgname);
  }
);

watch(
  () => form.value.pkgname,
  () => {
    if (props.mode === "add") autoDetectFromPkgname();
  }
);

async function handleSave() {
  const ok = await save(props.mode);
  if (ok) {
    emit("saved");
    emit("close");
  }
}
</script>

<template>
  <Modal :show="show" :title="mode === 'add' ? '添加软件包' : '编辑软件包'" width="720px" @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>
    <div class="form-two-col">
      <!-- 左栏：基本信息 -->
      <div class="form-col">
        <div class="col-section">
          <h4 class="col-title">基本信息</h4>
          <table class="form-table">
            <tbody>
              <tr v-if="mode === 'add'">
                <td class="label">包名</td>
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
                    <span>标记为需更新</span>
                  </label>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- 版本信息（只读） -->
        <div class="col-section" v-if="mode === 'edit' && detail">
          <h4 class="col-title">版本信息</h4>
          <table class="info-table">
            <tbody>
              <tr>
                <td class="label">AUR 版本</td>
                <td class="value">{{ detail.aur_version || '—' }}</td>
              </tr>
              <tr>
                <td class="label">AUR 更新时间</td>
                <td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td>
              </tr>
              <tr>
                <td class="label">上游版本</td>
                <td class="value">{{ detail.upstream_version || '—' }}</td>
              </tr>
              <tr>
                <td class="label">上次检查</td>
                <td class="value">{{ formatDateTime(detail.upstream_last_checked) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 右栏：配置 + 分类 -->
      <div class="form-col">
        <div class="col-section">
          <h4 class="col-title">检查配置</h4>
          <table class="form-table">
            <tbody>
              <tr>
                <td class="label">自动检查</td>
                <td>
                  <label class="checkbox-label">
                    <input type="checkbox" v-model="form.auto_check_enabled" />
                    <span>启用自动检查更新</span>
                  </label>
                </td>
              </tr>
              <tr>
                <td class="label">测试版本</td>
                <td>
                  <label class="checkbox-label">
                    <input type="checkbox" v-model="form.check_test_versions" />
                    <span>包含测试/预览版本</span>
                  </label>
                </td>
              </tr>
              <tr>
                <td class="label">二进制文件</td>
                <td>
                  <label class="checkbox-label">
                    <input type="checkbox" v-model="form.check_binary_files" />
                    <span>检查二进制文件版本</span>
                  </label>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="col-section">
          <h4 class="col-title">分类</h4>
          <table class="form-table">
            <tbody>
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
        </div>
      </div>
    </div>
    <template #footer>
      <button class="btn btn-outline" @click="emit('close')">取消</button>
      <button class="btn btn-primary" @click="handleSave" :disabled="saving || !canSaveBtn">
        {{ saving ? "保存中..." : "确认" }}
      </button>
    </template>
  </Modal>
</template>

<style scoped>
.form-two-col {
  display: flex;
  gap: 1.5rem;
  min-width: 640px;
}

.form-col {
  flex: 1;
  min-width: 0;
}

.col-section {
  margin-bottom: 1rem;
}

.col-section:last-child {
  margin-bottom: 0;
}

.col-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--accent);
  margin: 0 0 0.5rem 0;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--border);
}

.form-table .label {
  width: 90px;
}

.info-table .label {
  width: 100px;
}

.info-table td {
  padding: 0.375rem 0.5rem;
  font-size: 0.8125rem;
}
</style>
