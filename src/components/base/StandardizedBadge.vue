<script setup lang="ts">
const props = withDefaults(defineProps<{
  type?: "primary" | "secondary" | "success" | "warning" | "danger" | "info" | "neutral";
  size?: "sm" | "md" | "lg";
  variant?: "filled" | "outlined" | "soft";
  rounded?: boolean;
  dot?: boolean;
  text?: string;
}>(), {
  type: "neutral",
  size: "md",
  variant: "filled",
  rounded: true,
  dot: false,
});

const sizeClasses = {
  sm: "px-2 py-0.5 text-xs",
  md: "px-2.5 py-1 text-sm",
  lg: "px-3 py-1.5 text-sm",
};

const dotSizeClasses = {
  sm: "w-1.5 h-1.5",
  md: "w-2 h-2",
  lg: "w-2.5 h-2.5",
};

const typeClasses = {
  primary: {
    filled: "bg-blue-50 text-blue-600 border-blue-200",
    outlined: "bg-transparent text-blue-600 border-blue-300",
    soft: "bg-blue-50 text-blue-700",
  },
  secondary: {
    filled: "bg-gray-50 text-gray-600 border-gray-200",
    outlined: "bg-transparent text-gray-600 border-gray-300",
    soft: "bg-gray-50 text-gray-700",
  },
  success: {
    filled: "bg-green-50 text-green-600 border-green-200",
    outlined: "bg-transparent text-green-600 border-green-300",
    soft: "bg-green-50 text-green-700",
  },
  warning: {
    filled: "bg-yellow-50 text-yellow-600 border-yellow-200",
    outlined: "bg-transparent text-yellow-600 border-yellow-300",
    soft: "bg-yellow-50 text-yellow-700",
  },
  danger: {
    filled: "bg-red-50 text-red-600 border-red-200",
    outlined: "bg-transparent text-red-600 border-red-300",
    soft: "bg-red-50 text-red-700",
  },
  info: {
    filled: "bg-indigo-50 text-indigo-600 border-indigo-200",
    outlined: "bg-transparent text-indigo-600 border-indigo-300",
    soft: "bg-indigo-50 text-indigo-700",
  },
  neutral: {
    filled: "bg-slate-50 text-slate-600 border-slate-200",
    outlined: "bg-transparent text-slate-600 border-slate-300",
    soft: "bg-slate-50 text-slate-700",
  },
};

const baseClasses = "inline-flex items-center gap-1 font-medium border rounded-full transition-colors";
</script>

<template>
  <span
    :class="[
      baseClasses,
      sizeClasses[props.size],
      typeClasses[props.type]?.[props.variant],
      props.rounded ? 'rounded-full' : 'rounded',
    ]"
  >
    <span
      v-if="props.dot"
      :class="[
        'rounded-full',
        dotSizeClasses[props.size],
        typeClasses[props.type]?.filled.split(' ')[0].replace('bg-', 'bg-'),
      ]"
    ></span>
    <span>{{ props.text || $slots.default }}</span>
  </span>
</template>