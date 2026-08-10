import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'
import type { Invoke } from './useAuth'
import type { TemplateDto, TemplateValidationErrorDto } from '../types/bridge'

export type TemplateEditorStatus = 'idle' | 'loading' | 'saving' | 'saved' | 'error'

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)

function cloneTemplate(template: TemplateDto | null): TemplateDto | null {
  return template ? JSON.parse(JSON.stringify(template)) as TemplateDto : null
}

function sameTemplate(left: TemplateDto | null, right: TemplateDto | null): boolean {
  return JSON.stringify(left) === JSON.stringify(right)
}

export function useTemplateEditor(invoke: Invoke = defaultInvoke) {
  const current = ref<TemplateDto | null>(null)
  const initial = ref<TemplateDto | null>(null)
  const saved = ref<TemplateDto | null>(null)
  const status = ref<TemplateEditorStatus>('idle')
  const error = ref<string | null>(null)
  const validationErrors = ref<TemplateValidationErrorDto[]>([])
  const dirty = computed(() => !sameTemplate(current.value, saved.value))

  function setCurrent(template: TemplateDto | null) {
    current.value = cloneTemplate(template)
    initial.value = cloneTemplate(template)
    saved.value = cloneTemplate(template)
    status.value = template ? 'saved' : 'idle'
    error.value = null
    validationErrors.value = []
  }

  function startNew(data: Record<string, unknown> = {}) {
    current.value = { name: '', data: cloneTemplate({ name: '', data })?.data || {} }
    initial.value = null
    saved.value = null
    status.value = 'idle'
    error.value = null
    validationErrors.value = []
  }

  function updateDraft(template: TemplateDto) {
    current.value = cloneTemplate(template)
    error.value = null
    validationErrors.value = []
    if (status.value === 'saved') status.value = 'idle'
  }

  async function load(name: string) {
    if (dirty.value) {
      error.value = 'Save or reset the current template before opening another one.'
      status.value = 'error'
      return false
    }
    status.value = 'loading'
    error.value = null
    try {
      const template = await invoke<TemplateDto>('get_template', { name })
      setCurrent(template)
      return true
    } catch {
      status.value = 'error'
      error.value = 'Template could not be loaded.'
      return false
    }
  }

  async function validate() {
    if (!current.value) return false
    try {
      await invoke('validate_template', { data: current.value.data })
      validationErrors.value = []
      return true
    } catch (caught) {
      validationErrors.value = Array.isArray(caught) ? caught as TemplateValidationErrorDto[] : []
      error.value = validationErrors.value[0]?.message || 'Template validation failed.'
      status.value = 'error'
      return false
    }
  }

  async function save() {
    if (!current.value || !(await validate())) return false
    status.value = 'saving'
    error.value = null
    try {
      const command = saved.value ? 'update_template' : 'create_template'
      const result = await invoke<TemplateDto>(command, { data: current.value.data })
      current.value = cloneTemplate(result)
      initial.value = cloneTemplate(result)
      saved.value = cloneTemplate(result)
      status.value = 'saved'
      return true
    } catch {
      status.value = 'error'
      error.value = 'Template could not be saved.'
      return false
    }
  }

  function reset() {
    current.value = cloneTemplate(saved.value)
    error.value = null
    status.value = saved.value ? 'saved' : 'idle'
  }

  function discardChanges() {
    setCurrent(null)
  }

  return {
    current,
    initial,
    saved,
    status,
    error,
    validationErrors,
    dirty,
    setCurrent,
    startNew,
    updateDraft,
    load,
    validate,
    save,
    reset,
    discardChanges,
  }
}
