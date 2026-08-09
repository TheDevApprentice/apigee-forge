import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '../App.vue'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

describe('App M6-Bis flow', () => {
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

  it('starts Demo without OAuth or a network call', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return []
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.find('button.primary-action').exists()).toBe(false)
    expect(wrapper.text()).toContain('Demo operator')
    expect(invokeMock).not.toHaveBeenCalledWith('auth_login', undefined)
  })

  it('keeps Live behind the Login screen until OAuth succeeds', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'cloud', status: 'authentication_required', identity: null, organization: null, environment: null, error: null }
      if (command === 'auth_restore') return { authenticated: false }
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.find('button.primary-action').exists()).toBe(true)
    expect(wrapper.find('.sidebar').exists()).toBe(true)
    expect(wrapper.find('.connection-dot--connected').exists()).toBe(false)
    expect(wrapper.text()).toContain('Sign in with Google')
  })

  it('changes mode explicitly without starting OAuth', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'cloud', status: 'authentication_required', identity: null, organization: null, environment: null, error: null }
      if (command === 'auth_restore') return { authenticated: false }
      if (command === 'set_app_mode') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return []
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await wrapper.find('.mode-switcher select').setValue('demo')
    await flushPromises()

    expect(invokeMock).toHaveBeenCalledWith('set_app_mode', { mode: 'demo' })
    expect(wrapper.find('button.primary-action').exists()).toBe(false)
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

    await vi.waitFor(() => expect(wrapper.findAll('.workspace-selectors select')).toHaveLength(2))
    await vi.waitFor(() => expect(wrapper.text()).toContain('hello-world'))
    expect(invokeMock).toHaveBeenCalledWith('list_environments', { organization: 'org-one' })
    expect(invokeMock).toHaveBeenCalledWith('list_proxies', { organization: 'org-one', environment: 'test' })
  })
})
