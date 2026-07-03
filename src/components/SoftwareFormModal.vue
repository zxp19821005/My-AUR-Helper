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
import { watch, computed } from "vue";
import { useSoftwareForm, pkgTypes, checkerTypes } from "../composables/useSoftwareForm";
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

const {
  saving, error, form, licenses, languages,
  isDirty, canSave, save, init, autoDetectFromPkgname,
} = useSoftwareForm();

const dirty = computed(() => isDirty(props.pkgname ? { pkgname: props.pkgname } as any : null));
const canSaveBtn = computed(() => canSave(props.mode, dirty.value));

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
          <button class="btn btn-primary" @click="handleSave" :disabled="saving || !canSaveBtn">
            {{ saving ? "保存中..." : "确认" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
