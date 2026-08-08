import { ref } from 'vue'

export function useTemplateEditor() {
  const templateName = ref<string | null>(null)
  const dirty = ref(false)

  function reset() {
    templateName.value = null
    dirty.value = false
  }

  return { templateName, dirty, reset }
}
