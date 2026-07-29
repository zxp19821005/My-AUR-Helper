<!--
  ProxyToolbar.vue - 代理管理工具栏

  功能：
  - 搜索输入框
  - 类型筛选下拉框
  - 操作按钮：获取代理文件、解析代理文件、代理测试、删除选中
-->
<script setup lang="ts">
import { Trash2, Download, FileCode, Zap } from "@lucide/vue";
import StandardizedButton from "./base/StandardizedButton.vue";
import StandardizedSelect from "./base/StandardizedSelect.vue";
import StandardizedInput from "./base/StandardizedInput.vue";
import { PROXY_TYPE_OPTIONS } from "../composables/useProxyList";

defineProps<{
  searchQuery: string;
  typeFilter: string;
  loading: boolean;
  downloading: boolean;
  parsing: boolean;
  selectedCount: number;
}>();

const emit = defineEmits<{
  "update:searchQuery": [value: string];
  "update:typeFilter": [value: string];
  "download-proxy-file": [];
  "parse-proxy-file": [];
  "test-proxies": [];
  "delete-selected": [];
}>();
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <StandardizedInput
        :modelValue="searchQuery"
        @update:modelValue="emit('update:searchQuery', $event)"
        placeholder="搜索代理..."
        size="md"
        clearable
      />
    </div>
    <div class="toolbar-right">
      <StandardizedSelect
        :modelValue="typeFilter"
        @update:modelValue="emit('update:typeFilter', String($event))"
        size="sm"
      >
        <option v-for="opt in PROXY_TYPE_OPTIONS" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </StandardizedSelect>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="downloading"
        @click="emit('download-proxy-file')"
        title="获取代理文件"
      >
        <Download :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="parsing"
        @click="emit('parse-proxy-file')"
        title="解析代理文件"
      >
        <FileCode :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="outline"
        size="sm"
        :loading="loading"
        @click="emit('test-proxies')"
        title="代理测试"
      >
        <Zap :size="16" />
      </StandardizedButton>

      <StandardizedButton
        variant="danger"
        size="sm"
        :disabled="selectedCount === 0"
        @click="emit('delete-selected')"
        title="删除选中"
      >
        <Trash2 :size="16" />
      </StandardizedButton>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.toolbar-left {
  flex: 1;
  min-width: 200px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}
</style>