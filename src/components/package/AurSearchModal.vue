<!--
  AurSearchModal.vue - 从 AUR 搜索并导入新包对话框

  由 PackageList 通过 v-model:open 控制显隐；导入成功后 emit "imported"，
  由父组件负责刷新列表。内部自行管理搜索关键字 / 结果 / loading 状态。
-->
<script setup lang="ts">
import { ref } from "vue";
import * as softwareApi from "@/api/software";
import { addToast } from "@/composables/useToast";
import type { AurSearchResult } from "@/types";

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "imported"): void;
}>();

const keyword = ref("");
const results = ref<AurSearchResult[]>([]);
const searching = ref(false);
const importing = ref(false);

async function search() {
  const kw = keyword.value.trim();
  if (!kw) return;
  searching.value = true;
  try {
    results.value = await softwareApi.searchAurPackages(kw);
  } catch (error) {
    console.error("AUR 搜索失败:", error);
    addToast("error", `AUR 搜索失败: ${error}`);
  } finally {
    searching.value = false;
  }
}

/** 导入选中的 AUR 包 */
async function importPkg(result: AurSearchResult) {
  const pkgname = result.Name;
  // 默认包类型：-bin/-appimage → 二进制包，-git → Git仓库，其余 → 编译安装
  let packageType = 1; // Compiled
  if (pkgname.endsWith("-bin") || pkgname.endsWith("-appimage")) {
    packageType = 2; // Binary
  } else if (pkgname.endsWith("-git")) {
    packageType = 3; // Git
  }
  // 默认检查器类型：-bin 用 GitHub API（因为通常有 release），其他用手动
  let checkerType = 7; // Manual
  if (pkgname.endsWith("-bin")) {
    checkerType = 2; // GitHubAPI
  }

  importing.value = true;
  try {
    await softwareApi.importAurPackage({ pkgname, packageType, checkerType });
    addToast("success", `成功导入 ${pkgname}`);
    // 关闭对话框并通知父组件刷新列表
    emit("update:open", false);
    emit("imported");
  } catch (error) {
    console.error("AUR 导入失败:", error);
    addToast("error", `导入失败: ${error}`);
  } finally {
    importing.value = false;
  }
}

function close() {
  emit("update:open", false);
}
</script>

<template>
  <div v-if="open" class="aur-search-modal-overlay" @click.self="close">
    <div class="aur-search-modal">
      <div class="aur-search-modal-header">
        <h3>从 AUR 导入新包</h3>
        <button class="btn-close" @click="close">×</button>
      </div>
      <div class="aur-search-modal-body">
        <div class="aur-search-input-row">
          <input
            v-model="keyword"
            type="text"
            placeholder="输入包名搜索（如 electron45）..."
            class="aur-search-input"
            @keyup.enter="search"
          />
          <button
            class="btn btn-primary btn-sm"
            @click="search"
            :disabled="searching || !keyword.trim()"
          >
            <span v-if="searching">搜索中...</span>
            <span v-else>搜索</span>
          </button>
        </div>
        <div v-if="results.length > 0" class="aur-search-results">
          <div
            v-for="result in results"
            :key="result.Name"
            class="aur-search-result-item"
          >
            <div class="aur-result-info">
              <span class="aur-result-name">{{ result.Name }}</span>
              <span class="aur-result-version">{{ result.Version }}</span>
              <span class="aur-result-desc">{{ result.Description }}</span>
            </div>
            <button
              class="btn btn-sm btn-import"
              @click="importPkg(result)"
              :disabled="importing"
            >
              导入
            </button>
          </div>
        </div>
        <div v-else-if="!searching && results.length === 0 && keyword" class="aur-search-empty">
          未找到匹配的包
        </div>
        <div v-else-if="searching" class="aur-search-loading">
          搜索中...
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.aur-search-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.aur-search-modal {
  background: var(--bg, #fff);
  border-radius: 8px;
  width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.aur-search-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border, #e5e7eb);
}

.aur-search-modal-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text, #111827);
}

.btn-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: var(--text-muted, #6b7280);
  line-height: 1;
}

.btn-close:hover {
  color: var(--text, #111827);
}

.aur-search-modal-body {
  padding: 1rem 1.25rem;
  overflow-y: auto;
  flex: 1;
}

.aur-search-input-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.aur-search-input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 6px;
  font-size: 0.875rem;
  outline: none;
}

.aur-search-input:focus {
  border-color: var(--accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.btn-sm {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  white-space: nowrap;
}

.btn-primary {
  background: var(--accent, #3b82f6);
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover, #2563eb);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-import {
  background: var(--success, #22c55e);
  color: #fff;
  padding: 0.375rem 0.75rem;
}

.btn-import:hover:not(:disabled) {
  background: var(--success-hover, #16a34a);
}

.btn-import:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.aur-search-results {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.aur-search-result-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem;
  background: var(--muted-bg, #f9fafb);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 6px;
}

.aur-result-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.aur-result-name {
  font-weight: 600;
  color: var(--text, #111827);
  white-space: nowrap;
}

.aur-result-version {
  color: var(--accent, #3b82f6);
  font-size: 0.875rem;
  white-space: nowrap;
}

.aur-result-desc {
  color: var(--text-muted, #6b7280);
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.aur-search-empty,
.aur-search-loading {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted, #6b7280);
  font-size: 0.875rem;
}
</style>
