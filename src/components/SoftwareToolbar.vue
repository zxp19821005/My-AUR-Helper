<script setup lang="ts">
import { Edit, Trash2, RefreshCw, FileCode, GitBranch } from "@lucide/vue";

defineProps<{
  deleting: boolean;
  updatingAur: boolean;
  updatingPkgbuild: boolean;
  checking: boolean;
  disabled?: boolean;
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
      <button class="footer-btn" @click="emit('edit')" :disabled="disabled">
        <Edit :size="16" />
        <span>编辑</span>
      </button>
      <button class="footer-btn danger" @click="emit('delete')" :disabled="deleting || disabled">
        <Trash2 :size="16" />
        <span>删除</span>
      </button>
    </div>
    <div class="footer-right">
      <button class="footer-btn" @click="emit('updateAur')" :disabled="updatingAur || disabled">
        <GitBranch :size="16" />
        <span>{{ updatingAur ? '更新中...' : '更新 AUR 信息' }}</span>
      </button>
      <button class="footer-btn" @click="emit('updatePkgbuild')" :disabled="updatingPkgbuild || disabled">
        <FileCode :size="16" />
        <span>{{ updatingPkgbuild ? '更新中...' : '更新 PKGBUILD' }}</span>
      </button>
      <button class="footer-btn primary" @click="emit('checkUpdate')" :disabled="checking || disabled">
        <RefreshCw :size="16" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查上游版本' }}</span>
      </button>
    </div>
  </div>
</template>
