<script setup lang="ts">
defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  tone?: 'default' | 'danger'
}>()

const emit = defineEmits<{
  close: []
  confirm: []
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation" @click.self="emit('close')">
      <section class="base-modal" :class="{ 'base-modal--danger': tone === 'danger' }" role="dialog" aria-modal="true" aria-labelledby="modal-title" aria-describedby="modal-message">
        <div class="base-modal__header">
          <h2 id="modal-title">{{ title }}</h2>
          <button type="button" class="base-modal__close" aria-label="Close dialog" @click="emit('close')">×</button>
        </div>
        <p id="modal-message" class="base-modal__message">{{ message }}</p>
        <div class="base-modal__actions">
          <button type="button" @click="emit('close')">Cancel</button>
          <button type="button" class="base-modal__confirm" :class="{ 'base-modal__confirm--danger': tone === 'danger' }" @click="emit('confirm')">{{ confirmLabel || 'Confirm' }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
