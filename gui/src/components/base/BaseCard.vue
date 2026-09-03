<script setup lang="ts">
defineProps<{
  eyebrow?: string
}>()
</script>

<template>
  <section class="base-card">
    <!-- <p v-if="eyebrow" class="base-card__eyebrow">{{ eyebrow }}</p> -->
    <slot />
  </section>
</template>
