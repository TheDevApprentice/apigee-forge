<script setup lang="ts">
defineProps<{
  active?: boolean
  label: string
}>()
</script>

<template>
  <button class="base-button" :class="{ 'base-button--active': active }" :aria-label="label" :title="label">
    <slot />
  </button>
</template>
