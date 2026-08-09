import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '../App.vue'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

describe('App M6-03 flow', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('shows the safe offline login state without an Apigee account', async () => {
    invokeMock.mockResolvedValue({
      authenticated: false,
      mode: null,
      identity: null,
      project_id: null,
      selected_organization: null,
    })

    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.text()).toContain('Connect your Apigee workspace.')
    expect(wrapper.text()).toContain('Sign in with Google')
    expect(wrapper.find('button.primary-action').exists()).toBe(true)
  })

  it('loads context and proxies only after explicit selections', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'auth_status') return { authenticated: false }
      if (command === 'auth_login') {
        return {
          authenticated: true,
          mode: 'desktop',
          identity: 'developer@example.com',
          project_id: null,
          selected_organization: null,
        }
      }
      if (command === 'list_organizations') return [{ id: 'org-one', project_id: 'project-one', location: null }]
      if (command === 'list_environments') return [{ name: 'test' }]
      if (command === 'list_proxies') return [{ name: 'hello-world', revisions: [{ number: 1, deployed: false }] }]
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await new Promise((resolve) => setTimeout(resolve, 0))
    await vi.waitFor(() => expect((wrapper.find('button.primary-action').element as HTMLButtonElement).disabled).toBe(false))
    await wrapper.find('button.primary-action').trigger('click')
    await flushPromises()
    await flushPromises()

    await vi.waitFor(() => expect(wrapper.findAll('.context-grid select')).toHaveLength(2))
    const selects = wrapper.findAll('.context-grid select')
    expect(invokeMock).not.toHaveBeenCalledWith('list_proxies', expect.anything())

    await selects[0].setValue('org-one')
    await flushPromises()
    await selects[1].setValue('test')

    await vi.waitFor(() => expect(wrapper.text()).toContain('hello-world'))
    expect(invokeMock).toHaveBeenCalledWith('list_proxies', { organization: 'org-one', environment: 'test' })
  })
})
