<!--
  BaseFormModal.vue - 通用表单弹窗

  功能：
  - 收敛各模块的"新增/编辑"表单弹窗（如编程语言、License）
  - 通过 fields 配置驱动动态字段，避免每个枚举重复一套 modal 结构
  - 标题自动根据 mode + entityName 生成（新增xxx / 编辑xxx）
  - 打开时深拷贝 modelValue 作为本地草稿，保存时回传完整字段对象

  使用场景：LanguageManager、LicenseManager 等枚举管理页
-->
<script setup lang="ts">
import { ref, watch } from "vue";
import StandardizedModal from "./StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

export interface FormField {
  /** 字段键（对应 modelValue 的键） */
  key: string;
  /** 字段标签 */
  label: string;
  /** 输入框占位符 */
  placeholder?: string;
  /** 是否必填（默认 true，控制确认按钮可用性） */
  required?: boolean;
}

const props = withDefaults(defineProps<{
  /** 是否显示 */
  show: boolean;
  /** 模式：新增 / 编辑 */
  mode: "add" | "edit";
  /** 实体中文名，用于标题（如 "编程语言" → "新增编程语言"） */
  entityName: string;
  /** 字段配置 */
  fields: FormField[];
  /** 表单初始值（键需与 fields 对应） */
  modelValue: Record<string, string>;
}>(), {});

const emit = defineEmits<{
  save: [data: Record<string, string>];
  close: [];
}>();

const form = ref<Record<string, string>>({});

watch(() => props.show, (val) => {
  if (val) form.value = { ...props.modelValue };
});

/** 确认按钮是否可用：所有必填字段非空（required 默认 true，显式 false 时可空） */
const isValid = () =>
  props.fields.every(
    (f) => f.required === false || (form.value[f.key] ?? "").trim() !== ""
  );

function onSave() {
  emit("save", { ...form.value });
}
</script>

<template>
  <StandardizedModal
    :show="show"
    :title="mode === 'add' ? `新增${entityName}` : `编辑${entityName}`"
    width="sm"
    @close="emit('close')"
  >
    <div class="form-body">
      <div v-for="f in fields" :key="f.key" class="form-group">
        <label :for="`ff-${f.key}`">{{ f.label }}</label>
        <input
          :id="`ff-${f.key}`"
          v-model="form[f.key]"
          class="form-input"
          :placeholder="f.placeholder"
        />
      </div>
    </div>

    <template #footer>
      <StandardizedButton variant="outline" size="sm" @click="emit('close')">
        取消
      </StandardizedButton>
      <StandardizedButton
        variant="primary"
        size="sm"
        :disabled="!isValid()"
        @click="onSave"
      >
        确认
      </StandardizedButton>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.form-body {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.form-group label {
  font-size: 0.8125rem;
  color: var(--text-secondary);
}

.form-input {
  width: 100%;
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  box-sizing: border-box;
}

.form-input:focus {
  outline: none;
  border-color: var(--accent);
}
</style>
