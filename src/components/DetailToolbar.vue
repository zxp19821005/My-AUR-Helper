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
    <div class="footer-left">
      <button class="footer-btn" @click="emit('edit')" :disabled="loading">
        <Edit :size="14" /><span>编辑</span>
      </button>
      <button class="footer-btn danger" @click="emit('delete')" :disabled="deleting || loading">
        <Trash2 :size="14" /><span>{{ deleting ? '删除中...' : '删除' }}</span>
      </button>
    </div>
    <div class="footer-right">
      <button class="footer-btn" @click="emit('updateAur')" :disabled="updatingAur || loading">
        <GitBranch :size="14" /><span>{{ updatingAur ? '更新中...' : 'AUR 信息' }}</span>
      </button>
      <button class="footer-btn" @click="emit('updatePkgbuild')" :disabled="updatingPkgbuild || loading">
        <FileCode :size="14" /><span>{{ updatingPkgbuild ? '更新中...' : 'PKGBUILD' }}</span>
      </button>
      <button class="footer-btn primary" @click="emit('checkUpdate')" :disabled="checking || loading">
        <RefreshCw :size="14" :class="{ spinning: checking }" /><span>{{ checking ? '检查中...' : '检查上游' }}</span>
      </button>
    </div>
  </div>
</template>
