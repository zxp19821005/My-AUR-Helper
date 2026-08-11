<!--
  PackageActionButtons.vue - 软件包详情操作按钮组（编辑/同步PKGBUILD/更新AUR/检查上游/删除）

  被 DetailToolbar（弹窗底部）与 PackageDetailFooter（页面底部固定栏）复用，
  统一图标、语义色、加载态与分隔条，消除两处重复模板。

  按钮顺序（语义分组）：编辑 | 分隔条 | 同步PKGBUILD · 更新AUR · 检查上游 | 分隔条 | 删除
-->
<script setup lang="ts">
import { Icon } from "../../icons";
import StandardizedButton from "../base/StandardizedButton.vue";

withDefaults(
  defineProps<{
    loading?: boolean; // 全局只读锁：禁用全部按钮
    updatingAur?: boolean;
    updatingPkgbuild?: boolean;
    checking?: boolean;
    deleting?: boolean;
    size?: "sm" | "md"; // sm=2.25rem(弹窗) / md=2.5rem(底部栏)
    withDividers?: boolean; // 是否在编辑后、删除前插入分隔条
  }>(),
  {
    loading: false,
    updatingAur: false,
    updatingPkgbuild: false,
    checking: false,
    deleting: false,
    size: "sm",
    withDividers: true,
  }
);

const emit = defineEmits<{
  edit: [];
  delete: [];
  updateAur: [];
  updatePkgbuild: [];
  checkUpdate: [];
}>();
</script>

<template>
  <StandardizedButton
    class="tool-btn"
    :class="size"
    variant="outline"
    tone="info"
    :disabled="loading"
    title="编辑"
    @click="emit('edit')"
  >
    <component :is="Icon.actionEdit" :size="18" />
  </StandardizedButton>

  <div v-if="withDividers" class="toolbar-divider"></div>

  <StandardizedButton
    class="tool-btn"
    :class="size"
    variant="outline"
    tone="teal"
    :disabled="updatingPkgbuild || loading"
    title="同步 PKGBUILD"
    @click="emit('updatePkgbuild')"
  >
    <component :is="Icon.syncPkgbuild" :size="18" :class="{ spinning: updatingPkgbuild }" />
  </StandardizedButton>

  <StandardizedButton
    class="tool-btn"
    :class="size"
    variant="outline"
    tone="accent"
    :disabled="updatingAur || loading"
    title="更新 AUR 信息"
    @click="emit('updateAur')"
  >
    <component :is="Icon.syncAur" :size="18" :class="{ spinning: updatingAur }" />
  </StandardizedButton>

  <StandardizedButton
    class="tool-btn"
    :class="size"
    variant="outline"
    tone="success"
    :disabled="checking || loading"
    title="检查上游更新"
    @click="emit('checkUpdate')"
  >
    <component :is="Icon.actionSearch" :size="18" :class="{ spinning: checking }" />
  </StandardizedButton>

  <div v-if="withDividers" class="toolbar-divider"></div>

  <StandardizedButton
    class="tool-btn"
    :class="size"
    variant="outline"
    tone="danger"
    :disabled="deleting || loading"
    title="删除"
    @click="emit('delete')"
  >
    <component :is="Icon.actionDelete" :size="18" :class="{ spinning: deleting }" />
  </StandardizedButton>
</template>

<style scoped>
.toolbar-divider {
  width: 1px;
  height: 24px;
  background-color: var(--border);
  margin: 0 0.25rem;
}

/* 方形纯图标按钮：固定尺寸、图标居中 */
.tool-btn {
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  gap: 0;
}
.tool-btn.md {
  width: 2.5rem;
  height: 2.5rem;
}
</style>
