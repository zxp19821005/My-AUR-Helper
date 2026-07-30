<!--
  DetailToolbar.vue - 软件包详情页顶部操作工具栏

  功能：
  - 左侧：编辑、删除按钮
  - 右侧：更新 AUR 信息、同步 PKGBUILD、检查上游按钮
  - 按钮样式复用全局 toolbar-buttons.css
-->
<script setup lang="ts">
import { Edit, Trash2, RefreshCw, FileCode, GitBranch } from "@lucide/vue";

defineProps<{
  loading: boolean;
  updatingAur: boolean;
  updatingPkgbuild: boolean;
  checking: boolean;
  deleting: boolean;
}>();

const emit = defineEmits<{
  edit: [];
  delete: [];
  updateAur: [];
  updatePkgbuild: [];
  checkUpdate: [];
}>();
</script>

<template>
  <div class="footer-toolbar">
    <div class="toolbar-left">
      <button
        class="toolbar-btn btn-blue"
        @click="emit('edit')"
        :disabled="loading"
        title="编辑"
      >
        <Edit :size="16" />
        <span>编辑</span>
      </button>
      <button
        class="toolbar-btn btn-red"
        @click="emit('delete')"
        :disabled="deleting || loading"
        title="删除"
      >
        <Trash2 :size="16" />
        <span>{{ deleting ? '删除中...' : '删除' }}</span>
      </button>
    </div>
    <div class="toolbar-right">
      <div class="toolbar-divider"></div>
      <button
        class="toolbar-btn btn-purple"
        @click="emit('updateAur')"
        :disabled="updatingAur || loading"
        title="更新 AUR 信息"
      >
        <GitBranch :size="16" :class="{ spinning: updatingAur }" />
        <span>{{ updatingAur ? '更新中...' : 'AUR 信息' }}</span>
      </button>
      <button
        class="toolbar-btn btn-teal"
        @click="emit('updatePkgbuild')"
        :disabled="updatingPkgbuild || loading"
        title="同步 PKGBUILD"
      >
        <FileCode :size="16" :class="{ spinning: updatingPkgbuild }" />
        <span>{{ updatingPkgbuild ? '更新中...' : 'PKGBUILD' }}</span>
      </button>
      <button
        class="toolbar-btn btn-green"
        @click="emit('checkUpdate')"
        :disabled="checking || loading"
        title="检查上游更新"
      >
        <RefreshCw :size="16" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查上游' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.footer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1.25rem;
  background-color: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background-color: var(--border);
  margin: 0 0.25rem;
}
</style>
