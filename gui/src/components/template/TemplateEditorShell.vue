<script setup lang="ts">
type Step = 1 | 2 | 3 | 4

defineProps<{
  title: string
  step: Step
  nextLabel: string
  nextDisabled?: boolean
  showNext?: boolean
}>()

const emit = defineEmits<{
  back: []
  next: []
}>()
</script>

<template>
  <section class="template-editor-shell" aria-label="Template editor">
    <header class="template-editor-shell__header">
      <div>
        <span class="template-workspace__eyebrow">Editing template</span>
        <h2>{{ title || 'New template' }}</h2>
        <p>Complete each step, validate your template, then save it locally.</p>
      </div>
      <button type="button" class="template-editor-shell__back" @click="emit('back')">Back to templates</button>
    </header>
    <nav class="template-steps" aria-label="Template editing steps">
      <span v-for="item in [{ number: 1, label: 'Details' }, { number: 2, label: 'Flow' }, { number: 3, label: 'Policies' }, { number: 4, label: 'Save' }]" :key="item.number" class="template-step" :class="{ 'template-step--active': step === item.number, 'template-step--complete': step > item.number }"><b>{{ item.number }}</b>{{ item.label }}</span>
    </nav>
    <div class="template-editor-shell__content"><slot /></div>
    <footer v-if="showNext !== false" class="template-editor-shell__actions">
      <button type="button" :disabled="step === 1" @click="emit('back')">Back</button>
      <button type="button" class="primary-action" :disabled="nextDisabled" @click="emit('next')">{{ nextLabel }}</button>
    </footer>
  </section>
</template>
