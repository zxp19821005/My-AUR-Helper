<!--
  DetailToolbar.vue - 软件包详情弹窗底部操作工具栏

  仅作为 PackageActionButtons 的薄包装，提供统一的居中布局（.footer-toolbar）。
  按钮组的图标、语义色、加载态与事件已由 PackageActionButtons 统一实现。
-->
<script setup lang="ts">
import PackageActionButtons from "./PackageActionButtons.vue";

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
    <PackageActionButtons
      :loading="loading"
      :updating-aur="updatingAur"
      :updating-pkgbuild="updatingPkgbuild"
      :checking="checking"
      :deleting="deleting"
      size="sm"
      @edit="emit('edit')"
      @delete="emit('delete')"
      @update-aur="emit('updateAur')"
      @update-pkgbuild="emit('updatePkgbuild')"
      @check-update="emit('checkUpdate')"
    />
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
</style>
