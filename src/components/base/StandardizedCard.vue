<template>
  <div class="standardized-card" :class="[variant, layout]">
    <div class="card-header" v-if="title || subtitle">
      <div class="card-title-section">
        <h3 class="card-title" v-if="title">{{ title }}</h3>
        <p class="card-subtitle" v-if="subtitle">{{ subtitle }}</p>
      </div>
      <div class="card-status" v-if="$slots.status">
        <slot name="status"></slot>
      </div>
    </div>
    <div class="card-content">
      <slot></slot>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  title: {
    type: String,
    default: '',
  },
  subtitle: {
    type: String,
    default: '',
  },
  variant: {
    type: String,
    default: '',
    validator: (value) => ['', 'variant-compact', 'variant-wide'].includes(value),
  },
  layout: {
    type: String,
    default: '',
    validator: (value) => ['', 'layout-table', 'layout-flow'].includes(value),
  },
})
</script>

<style scoped>
.standardized-card {
  background: var(--bg-card);
  border-radius: 8px;
  border: 1px solid var(--border);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  padding: 1.25rem;
  transition: all 0.2s;
}

.standardized-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  transform: translateY(-1px);
}

.standardized-card.variant-compact {
  padding: 0.75rem;
}

.standardized-card.variant-wide {
  padding: 1.5rem;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--border);
}

.card-title-section {
  flex: 1;
}

.card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.card-subtitle {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-top: 0.25rem;
}

.card-status {
  flex-shrink: 0;
  margin-left: 1rem;
}

.card-content {
  font-size: 0.875rem;
  color: var(--text-primary);
}

/* Table layout specific styles */
.standardized-card.layout-table .card-content {
  margin-top: 0.75rem;
}

/* Flow layout specific styles */
.standardized-card.layout-flow {
  display: flex;
  flex-direction: column;
}
</style>