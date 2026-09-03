<script setup lang="ts">
defineProps<{ label: string }>()
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <button type="button" class="base-back-button" @click="emit('click')">
    <span class="base-back-button__icon" aria-hidden="true">
      <svg viewBox="0 0 20 20" focusable="false"><path d="M11.5 4.5 6 10l5.5 5.5M6.5 10H15" /></svg>
    </span>
    <span>{{ label }}</span>
  </button>
</template>
