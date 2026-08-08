<script setup lang="ts">
defineProps<{
  label: string
}>()
</script>

<template>
  <span class="base-chip">{{ label }}</span>
</template>
