import { describe, expect, it, vi } from 'vitest'
import { useTemplateEditor, type TemplateEditorStatus } from '../composables/useTemplateEditor'
import type { Invoke } from '../composables/useAuth'
import type { TemplateDto } from '../types/bridge'

const template: TemplateDto = {
  name: 'orders',
  data: {
    metadata: { name: 'orders', owner: 'platform' },
    flow: { pre_flow: {}, post_flow: {} },
  },
}

describe('useTemplateEditor', () => {
  it('tracks saved and dirty states and can reset the draft', () => {
    const editor = useTemplateEditor(vi.fn() as Invoke)

    editor.setCurrent(template)
    expect(editor.dirty.value).toBe(false)
    editor.updateDraft({ ...template, data: { ...template.data, changed: true } })
    expect(editor.dirty.value).toBe(true)
    editor.reset()
    expect(editor.dirty.value).toBe(false)
    expect(editor.status.value).toBe('saved')
  })

  it('prevents loading another template while the current draft is dirty', async () => {
    const invoke = vi.fn().mockResolvedValue(template) as unknown as Invoke
    const editor = useTemplateEditor(invoke)
    editor.setCurrent(template)
    editor.updateDraft({ ...template, data: { changed: true } })

    const loaded = await editor.load('other')

    expect(loaded).toBe(false)
    expect(editor.status.value as TemplateEditorStatus).toBe('error')
    expect(editor.error.value).toContain('Save or reset')
    expect(invoke).not.toHaveBeenCalled()
  })

  it('saves a new draft and records the saved snapshot', async () => {
    const invoke = vi.fn().mockResolvedValue(template) as unknown as Invoke
    const editor = useTemplateEditor(invoke)
    editor.startNew(template.data)
    editor.updateDraft(template)

    const saved = await editor.save()

    expect(saved).toBe(true)
    expect(editor.dirty.value).toBe(false)
    expect(editor.status.value).toBe('saved')
    expect(invoke).toHaveBeenCalledWith('create_template', { data: template.data })
  })
})
