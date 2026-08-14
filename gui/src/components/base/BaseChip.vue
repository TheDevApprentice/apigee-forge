<script setup lang="ts">
defineProps<{
  label: string
  tone?: 'accent' | 'success' | 'warning' | 'danger' | 'neutral'
}>()
</script>

<template>
  <span class="base-chip" :class="`base-chip--${tone || 'accent'}`">{{ label }}</span>
</template>
