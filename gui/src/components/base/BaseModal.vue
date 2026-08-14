<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

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

const dialog = ref<HTMLElement | null>(null)
const confirmButton = ref<HTMLButtonElement | null>(null)
let previouslyFocused: HTMLElement | null = null

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close')
}

onMounted(() => {
  previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null
  window.addEventListener('keydown', onKeydown)
  void nextTick(() => confirmButton.value?.focus())
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  previouslyFocused?.focus()
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation" @click.self="emit('close')">
      <section ref="dialog" class="base-modal" :class="{ 'base-modal--danger': tone === 'danger' }" role="dialog" aria-modal="true" aria-labelledby="modal-title" aria-describedby="modal-message" @keydown.esc="emit('close')">
        <div class="base-modal__header">
          <h2 id="modal-title">{{ title }}</h2>
          <button type="button" class="base-modal__close" aria-label="Close dialog" @click="emit('close')">×</button>
        </div>
        <p id="modal-message" class="base-modal__message">{{ message }}</p>
        <div class="base-modal__actions">
          <button type="button" @click="emit('close')">Cancel</button>
          <button ref="confirmButton" type="button" class="base-modal__confirm" :class="{ 'base-modal__confirm--danger': tone === 'danger' }" @click="emit('confirm')">{{ confirmLabel || 'Confirm' }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
