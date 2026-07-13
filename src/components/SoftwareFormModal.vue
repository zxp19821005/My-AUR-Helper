<!--
  SoftwareFormModal.vue - 软件包添加/编辑弹窗

  功能：
  - 提供软件包的添加和编辑表单
  - 支持包名、上游地址、软件类型、检查器类型等配置
  - 支持 License 可搜索下拉选择
  - 支持编程语言多选

  Props:
  - show: boolean - 是否显示弹窗
  - mode: 'add' | 'edit' - 模式
  - pkgname: string - 编辑时的包名

  Events:
  - close: 关闭弹窗
  - saved: 保存成功
-->
<script setup lang="ts">
import { watch, computed, ref } from "vue";
import { useSoftwareForm, pkgTypes, checkerTypes } from "../composables/useSoftwareForm";
import { useLicenseSelect } from "../composables/useLicenseSelect";
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

function toggleLanguage(langId: number) {
  const idx = form.value.language_ids.indexOf(langId);
  if (idx >= 0) {
    form.value.language_ids.splice(idx, 1);
  } else {
    form.value.language_ids.push(langId);
  }
}

const searchableSelectRef = ref<HTMLElement | null>(null);

const {
  licenseSearch,
  licenseDropdownOpen,
  filteredLicenses,
  selectLicense,
  getSelectedLicenseLabel,
} = useLicenseSelect(licenses, computed(() => form.value.license_ids), searchableSelectRef);

watch(
  () => props.show,
  (val) => { if (val) init(props.mode, props.pkgname); }
);

watch(
  () => form.value.pkgname,
  () => { if (props.mode === "add") autoDetectFromPkgname(); }
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
  <Modal :show="show" :title="mode === 'add' ? '添加软件包' : '编辑软件包'" width="600px" @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>
    <div class="form-container">
      <div class="form-row-full">
        <span v-if="mode === 'edit'" class="pkgname-text">{{ form.pkgname }}</span>
        <input v-else v-model="form.pkgname" class="form-input pkgname-input" placeholder="输入包名" />
      </div>

      <div class="form-row-full">
        <label class="form-label">上游地址</label>
        <input v-model="form.upstream_url" class="form-input" placeholder="https://..." />
      </div>

      <div class="form-row-full">
        <label class="form-label">软件类型</label>
        <div class="radio-group">
          <label v-for="t in pkgTypes" :key="t.id" class="radio-item">
            <input type="radio" :value="t.id" v-model.number="form.package_type_id" />
            <span>{{ t.label }}</span>
          </label>
        </div>
      </div>

      <div class="form-row-full">
        <label class="form-label">检查器类型</label>
        <div class="radio-group">
          <label v-for="c in checkerTypes" :key="c.id" class="radio-item">
            <input type="radio" :value="c.id" v-model.number="form.checker_type_id" />
            <span>{{ c.label }}</span>
          </label>
        </div>
      </div>

      <div class="form-row-full">
        <label class="form-label">编程语言</label>
        <div class="checkbox-group">
          <label v-for="lang in languages" :key="lang.id ?? `lang-${lang.name}`" class="checkbox-item">
            <input type="checkbox" :checked="lang.id !== null && form.language_ids.includes(lang.id)" @change="lang.id !== null && toggleLanguage(lang.id)" />
            <span>{{ lang.name }}</span>
          </label>
        </div>
      </div>

      <div class="form-row-full">
        <label class="form-label">版本提取关键字</label>
        <input v-model="form.version_extract_regex" class="form-input" placeholder="输入正则表达式，如 v?(\d+\.\d+\.\d+)" />
        <span class="form-hint">用于自定义版本号提取规则，支持正则表达式语法</span>
      </div>

      <div class="form-row-inline">
        <div class="inline-item">
          <label class="checkbox-label" v-if="mode === 'edit'">
            <input type="checkbox" v-model="form.is_outdated" />
            <span>状态：需更新</span>
          </label>
        </div>
        <div class="inline-item">
          <label class="checkbox-label">
            <input type="checkbox" v-model="form.auto_check_enabled" />
            <span>自动检查</span>
          </label>
        </div>
      </div>

      <div class="form-row-inline">
        <div class="inline-item">
          <label class="checkbox-label">
            <input type="checkbox" v-model="form.check_test_versions" />
            <span>测试版本</span>
          </label>
        </div>
        <div class="inline-item">
          <label class="checkbox-label">
            <input type="checkbox" v-model="form.check_binary_files" />
            <span>二进制文件</span>
          </label>
        </div>
      </div>

      <div class="form-row-full">
        <label class="form-label">License</label>
        <div ref="searchableSelectRef" class="searchable-select" @click="licenseDropdownOpen = !licenseDropdownOpen">
          <div class="select-display">
            <span>{{ getSelectedLicenseLabel() }}</span>
            <span class="select-arrow">▼</span>
          </div>
          <div v-if="licenseDropdownOpen" class="select-dropdown">
            <input v-model="licenseSearch" class="select-search-input" placeholder="搜索 License..." @click.stop />
            <div class="select-options">
              <div class="select-option" @click.stop="selectLicense(null)">未设置</div>
              <div v-for="lic in filteredLicenses" :key="lic.id ?? `lic-${lic.spdx_id}`" class="select-option" @click.stop="selectLicense(lic.spdx_id)">
                {{ lic.spdx_id }} — {{ lic.full_name }}
              </div>
            </div>
          </div>
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
.form-container { display: flex; flex-direction: column; gap: 0.75rem; }
.form-row-full { display: flex; flex-direction: column; gap: 0.375rem; }
.form-row-inline { display: flex; align-items: center; gap: 1.5rem; }
.inline-item { display: flex; align-items: center; }
.form-label { font-size: 0.8125rem; font-weight: 600; color: var(--text-secondary); }
.pkgname-text { font-size: 1.125rem; font-weight: 700; color: var(--accent); text-align: center; }
.pkgname-input { text-align: center; font-size: 1rem; font-weight: 600; }
.radio-group { display: flex; flex-wrap: wrap; gap: 0.5rem; }
.radio-item {
  display: inline-flex; align-items: center; gap: 0.375rem;
  padding: 0.375rem 0.625rem; border-radius: 6px; border: 1px solid var(--border);
  cursor: pointer; font-size: 0.8125rem; color: var(--text-primary); transition: all 0.15s ease;
}
.radio-item:hover { border-color: var(--accent); background: rgba(108, 99, 255, 0.08); }
.radio-item:has(input:checked) { border-color: var(--accent); background: rgba(108, 99, 255, 0.15); color: var(--accent); }
.radio-item input[type="radio"] { display: none; }
.checkbox-group { display: flex; flex-wrap: wrap; gap: 0.5rem; }
.checkbox-item {
  display: inline-flex; align-items: center; gap: 0.375rem;
  padding: 0.375rem 0.625rem; border-radius: 6px; border: 1px solid var(--border);
  cursor: pointer; font-size: 0.8125rem; color: var(--text-primary); transition: all 0.15s ease;
}
.checkbox-item:hover { border-color: var(--accent); background: rgba(108, 99, 255, 0.08); }
.checkbox-item:has(input:checked) { border-color: var(--accent); background: rgba(108, 99, 255, 0.15); color: var(--accent); }
.checkbox-item input[type="checkbox"] { display: none; }
.searchable-select { position: relative; width: 100%; }
.select-display {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.375rem 0.625rem; border-radius: 6px; border: 1px solid var(--border);
  background-color: var(--bg-primary); color: var(--text-primary);
  font-size: 0.8125rem; cursor: pointer; transition: border-color 0.15s ease;
}
.select-display:hover { border-color: var(--accent); }
.select-arrow { font-size: 0.625rem; color: var(--text-secondary); transition: transform 0.15s ease; }
.select-dropdown {
  position: absolute; top: 100%; left: 0; right: 0; margin-top: 0.25rem;
  background: var(--bg-secondary); border: 1px solid var(--border);
  border-radius: 8px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  z-index: 100; overflow: hidden;
}
.select-search-input {
  width: 100%; padding: 0.5rem 0.75rem; border: none; border-bottom: 1px solid var(--border);
  background: var(--bg-primary); color: var(--text-primary); font-size: 0.8125rem; outline: none;
}
.select-search-input:focus { border-bottom-color: var(--accent); }
.select-options { max-height: 200px; overflow-y: auto; }
.select-option {
  padding: 0.5rem 0.75rem; font-size: 0.8125rem; color: var(--text-primary);
  cursor: pointer; transition: background 0.1s ease;
}
.select-option:hover { background: rgba(108, 99, 255, 0.1); }
.select-option.selected { background: rgba(108, 99, 255, 0.2); color: var(--accent); }
.form-input {
  padding: 0.375rem 0.625rem; border-radius: 6px; border: 1px solid var(--border);
  background-color: var(--bg-primary); color: var(--text-primary); font-size: 0.8125rem; width: 100%;
}
.form-input:focus { outline: none; border-color: var(--accent); }
.form-hint { font-size: 0.75rem; color: var(--text-secondary); }
.checkbox-label {
  display: inline-flex; align-items: center; gap: 0.5rem;
  cursor: pointer; font-size: 0.875rem; color: var(--text-primary);
}
.checkbox-label input[type="checkbox"] {
  width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer;
}
</style>