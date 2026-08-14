<script setup lang="ts">
defineProps<{
  active?: boolean
  disabled?: boolean
  label: string
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
}>()
</script>

<template>
  <button
    class="base-button"
    :class="[
      `base-button--${variant || 'ghost'}`,
      { 'base-button--active': active },
    ]"
    :aria-label="label"
    :title="label"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>
