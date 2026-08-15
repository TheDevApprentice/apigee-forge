<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{
  open: boolean
  eyebrow: string
  title: string
  closeLabel?: string
}>()

const emit = defineEmits<{
  close: []
}>()

const closeButton = ref<HTMLButtonElement | null>(null)
let previouslyFocused: HTMLElement | null = null

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close')
}

watch(() => props.open, (open) => {
  if (open) {
    previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null
    window.addEventListener('keydown', onKeydown)
    void nextTick(() => closeButton.value?.focus())
  } else {
    window.removeEventListener('keydown', onKeydown)
    previouslyFocused?.focus()
    previouslyFocused = null
  }
}, { immediate: true })

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  previouslyFocused?.focus()
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="base-drawer-backdrop" role="presentation" @click.self="emit('close')">
      <aside class="base-drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title" @keydown.esc="emit('close')">
        <div class="base-drawer__header">
          <div>
            <p class="base-card__eyebrow">{{ eyebrow }}</p>
            <h2 id="drawer-title">{{ title }}</h2>
          </div>
          <button ref="closeButton" type="button" class="base-drawer__close" :aria-label="closeLabel || 'Close details'" @click="emit('close')">×</button>
        </div>
        <div class="base-drawer__body">
          <slot />
        </div>
      </aside>
    </div>
  </Teleport>
</template>
