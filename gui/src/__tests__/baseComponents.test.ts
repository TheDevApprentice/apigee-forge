import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import BaseButton from '../components/base/BaseButton.vue'
import BaseChip from '../components/base/BaseChip.vue'
import BaseModal from '../components/base/BaseModal.vue'
import CollectionList from '../components/base/CollectionList.vue'


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

  it('configures a reusable collection list with search and filter events', async () => {
    const wrapper = mount(CollectionList, {
      props: {
        title: 'Proxy catalogue',
        searchValue: '',
        filters: [{ value: 'all', label: 'All', count: 2 }, { value: 'live', label: 'Live', count: 1 }],
      },
    })

    await wrapper.get('.collection-list__search').setValue('orders')
    await wrapper.findAll('.collection-list__filter')[1].trigger('click')

    expect(wrapper.emitted('update:searchValue')?.[0]).toEqual(['orders'])
    expect(wrapper.emitted('update:activeFilter')?.[0]).toEqual(['live'])
  })

  it('provides modal dialog descriptions and a stable confirm hook', async () => {
    const wrapper = mount(BaseModal, {
      props: { open: true, title: 'Confirm deployment', message: 'Review the target first.' },
      attachTo: document.body,
    })

    expect(document.querySelector('[role="dialog"]')?.getAttribute('aria-describedby')).toBe('modal-message')
    expect(document.querySelector('.base-modal__confirm')).not.toBeNull()
    await flushPromises()
    expect(document.activeElement).toBe(document.querySelector('.base-modal__confirm'))
    await wrapper.unmount()
  })
})
