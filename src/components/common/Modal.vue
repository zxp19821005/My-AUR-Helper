<script setup lang="ts">
defineProps<{
  show: boolean;
  title?: string;
  width?: string;
  hideHeader?: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal" :style="width ? { maxWidth: width, minWidth: width } : {}">
        <div v-if="!hideHeader" class="modal-header">
          <h3 v-if="title">{{ title }}</h3>
          <slot name="header"></slot>
          <button class="modal-close" @click="emit('close')">
            <slot name="close">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </slot>
          </button>
        </div>
        <div v-if="$slots.error" class="modal-error">
          <slot name="error"></slot>
        </div>
        <div class="modal-body">
          <slot></slot>
        </div>
        <div v-if="$slots.footer" class="modal-footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </div>
  </Teleport>
</template>
