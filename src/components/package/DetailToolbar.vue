<!--
  DetailToolbar.vue - 软件包详情弹窗底部操作工具栏

  功能：
  - 图标按钮顺序：编辑 | 分隔符 | 同步PKGBUILD | 更新AUR | 更新上游 | 分隔符 | 删除
  - 按钮样式复用全局 toolbar-buttons.css，本组件额外覆盖为纯图标按钮
  - 工具栏整体居中排布，避免编辑/删除分居两端造成中间大段空白
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
    <!-- 顺序：编辑 | 分隔符 | 同步PKGBUILD | 更新AUR | 更新上游 | 分隔符 | 删除 -->
    <button
      class="toolbar-btn btn-blue"
      @click="emit('edit')"
      :disabled="loading"
      title="编辑"
    >
      <Edit :size="18" />
    </button>

    <div class="toolbar-divider"></div>

    <button
      class="toolbar-btn btn-teal"
      @click="emit('updatePkgbuild')"
      :disabled="updatingPkgbuild || loading"
      title="同步 PKGBUILD"
    >
      <FileCode :size="18" :class="{ spinning: updatingPkgbuild }" />
    </button>

    <button
      class="toolbar-btn btn-purple"
      @click="emit('updateAur')"
      :disabled="updatingAur || loading"
      title="更新 AUR 信息"
    >
      <GitBranch :size="18" :class="{ spinning: updatingAur }" />
    </button>

    <button
      class="toolbar-btn btn-green"
      @click="emit('checkUpdate')"
      :disabled="checking || loading"
      title="更新上游信息"
    >
      <RefreshCw :size="18" :class="{ spinning: checking }" />
    </button>

    <div class="toolbar-divider"></div>

    <button
      class="toolbar-btn btn-red"
      @click="emit('delete')"
      :disabled="deleting || loading"
      title="删除"
    >
      <Trash2 :size="18" :class="{ spinning: deleting }" />
    </button>
  </div>
</template>

<style scoped>
.footer-toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  background-color: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.toolbar-divider {
  width: 1px;
  height: 24px;
  background-color: var(--border);
  margin: 0 0.25rem;
}

/* 纯图标按钮：覆盖全局 .toolbar-btn 的文字按钮样式 */
.toolbar-btn {
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  justify-content: center;
  gap: 0;
}
</style>
