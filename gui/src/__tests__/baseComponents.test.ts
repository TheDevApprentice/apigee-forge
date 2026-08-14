import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import BaseButton from '../components/base/BaseButton.vue'
import BaseChip from '../components/base/BaseChip.vue'
import BaseModal from '../components/base/BaseModal.vue'


describe('base components', () => {
  it('exposes button variants, disabled state and accessible label', () => {
    const wrapper = mount(BaseButton, {
      props: { label: 'Save draft', variant: 'primary', disabled: true },
    })
    const button = wrapper.get('button')

    expect(button.classes()).toContain('base-button--primary')
    expect(button.attributes('aria-label')).toBe('Save draft')
    expect((button.element as HTMLButtonElement).disabled).toBe(true)
  })

  it('exposes semantic chip tones without changing the label contract', () => {
    const wrapper = mount(BaseChip, { props: { label: 'Not deployed', tone: 'warning' } })

    expect(wrapper.classes()).toContain('base-chip--warning')
    expect(wrapper.text()).toBe('Not deployed')
  })

  it('provides modal dialog descriptions and a stable confirm hook', async () => {
    const wrapper = mount(BaseModal, {
      props: { open: true, title: 'Confirm deployment', message: 'Review the target first.' },
      attachTo: document.body,
    })

    expect(document.querySelector('[role="dialog"]')?.getAttribute('aria-describedby')).toBe('modal-message')
    expect(document.querySelector('.base-modal__confirm')).not.toBeNull()
    await wrapper.unmount()
  })
})
