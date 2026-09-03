<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref } from 'vue'

type SelectOption = { value: string; label: string; description?: string }

const props = withDefaults(defineProps<{
  modelValue: string
  options: SelectOption[]
  placeholder?: string
  disabled?: boolean
  label: string
}>(), { placeholder: 'Select an option', disabled: false })

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const open = ref(false)
const trigger = ref<HTMLButtonElement | null>(null)
const list = ref<HTMLElement | null>(null)
const selected = () => props.options.find((option) => option.value === props.modelValue)

function closeFromAnotherSelect(event: Event) {
  if ((event as CustomEvent<HTMLButtonElement | null>).detail !== trigger.value) open.value = false
}

function toggle() {
  if (props.disabled) return
  open.value = !open.value
  if (open.value) {
    window.dispatchEvent(new CustomEvent('base-select-open', { detail: trigger.value }))
    void nextTick(() => list.value?.querySelector<HTMLElement>('[aria-selected="true"]')?.focus())
  }
}

function choose(option: SelectOption) {
  emit('update:modelValue', option.value)
  open.value = false
  void nextTick(() => trigger.value?.focus())
}

function closeOnOutside(event: MouseEvent) {
  if (!(event.target instanceof Node) || !trigger.value?.parentElement?.contains(event.target)) open.value = false
}

document.addEventListener('click', closeOnOutside)
window.addEventListener('base-select-open', closeFromAnotherSelect)
onBeforeUnmount(() => {
  document.removeEventListener('click', closeOnOutside)
  window.removeEventListener('base-select-open', closeFromAnotherSelect)
})
</script>

<template>
  <div class="base-select" :class="{ 'base-select--open': open, 'base-select--disabled': disabled }">
    <button ref="trigger" type="button" class="base-select__trigger" :aria-label="label" :aria-expanded="open" :disabled="disabled" @click.stop="toggle">
      <span class="base-select__value">{{ selected()?.label || placeholder }}</span>
      <span class="base-select__chevron" aria-hidden="true">⌄</span>
    </button>
    <div v-if="open" ref="list" class="base-select__menu" role="listbox" :aria-label="label">
      <button v-for="option in options" :key="option.value" type="button" class="base-select__option" :class="{ 'base-select__option--selected': option.value === modelValue }" role="option" :aria-selected="option.value === modelValue" @click="choose(option)">
        <span class="base-select__option-check" aria-hidden="true">{{ option.value === modelValue ? '✓' : '' }}</span>
        <span><strong>{{ option.label }}</strong><small v-if="option.description">{{ option.description }}</small></span>
      </button>
      <span v-if="!options.length" class="base-select__empty">No options available</span>
    </div>
  </div>
</template>
