<script setup lang="ts">
import { ArrowLeft, ArrowRight } from "@lucide/vue";

defineProps<{
  prev: string | null;
  next: string | null;
}>();

const emit = defineEmits<{
  navigate: [direction: "prev" | "next"];
}>();
</script>

<template>
  <div class="floating-nav">
    <button class="nav-btn" :class="{ disabled: !prev }" @click.stop="emit('navigate', 'prev')" title="上一个">
      <ArrowLeft :size="20" />
    </button>
    <button class="nav-btn" :class="{ disabled: !next }" @click.stop="emit('navigate', 'next')" title="下一个">
      <ArrowRight :size="20" />
    </button>
  </div>
</template>

<style scoped>
.floating-nav {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  transform: translateY(-50%);
  display: flex;
  justify-content: space-between;
  pointer-events: none;
  padding: 0 0.5rem;
}

.floating-nav .nav-btn {
  pointer-events: auto;
  width: 36px;
  height: 36px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
  backdrop-filter: blur(4px);
}

.floating-nav .nav-btn:hover:not(.disabled) {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
  transform: scale(1.1);
}

.floating-nav .nav-btn.disabled { opacity: 0.3; cursor: not-allowed; }
</style>
