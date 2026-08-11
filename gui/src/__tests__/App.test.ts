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
    expect(wrapper.text()).toContain('Demo workspace')
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

  it('opens Create proxy in Proxies with a template selector', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return [{ id: 'demo-org', project_id: 'demo-project', location: null }]
      if (command === 'list_environments') return [{ name: 'demo' }]
      if (command === 'list_templates') return [{
        name: 'orders',
        data: {
          metadata: { name: 'orders', owner: 'platform', naming_convention: { prefix: 'api-', case: 'kebab-case' } },
          flow: { pre_flow: { request: [], response: [] }, post_flow: { request: [], response: [] } },
        },
      }]
      if (command === 'list_proxies') return []
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await vi.waitFor(() => expect(wrapper.findAll('.dashboard-action-card')).toHaveLength(2))
    await wrapper.findAll('.dashboard-action-card')[1].trigger('click')
    await vi.waitFor(() => expect(wrapper.find('select[aria-label="Select proxy template"]').exists()).toBe(true))

    await wrapper.find('select[aria-label="Select proxy template"]').setValue('orders')
    expect(wrapper.text()).toContain('Prepare proxy creation')
    expect(wrapper.text()).toContain('api-orders')
  })

  it('prepares a non-mutating proxy creation preview from a saved template', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return [{ id: 'demo-org', project_id: 'demo-project', location: null }]
      if (command === 'list_environments') return [{ name: 'demo' }]
      if (command === 'list_templates') return [{
        name: 'orders',
        data: {
          metadata: { name: 'orders', owner: 'platform', naming_convention: { prefix: 'api-', case: 'kebab-case' } },
          flow: { pre_flow: { request: [], response: [] }, post_flow: { request: [], response: [] } },
        },
      }]
      if (command === 'list_proxies') return []
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await wrapper.find('button[aria-label="Templates"]').trigger('click')
    await flushPromises()
    await vi.waitFor(() => expect(invokeMock).toHaveBeenCalledWith('list_templates'))
    await vi.waitFor(() => expect(wrapper.find('.template-list__prepare').exists()).toBe(true))

    await wrapper.find('.template-list__prepare').trigger('click')
    expect(wrapper.text()).toContain('Prepare proxy creation')
    expect(wrapper.text()).toContain('api-orders')
    expect(wrapper.text()).toContain('Provide a name for the OpenAPI specification.')

    await wrapper.find('input[placeholder="openapi.yaml"]').setValue('orders.yaml')
    await wrapper.find('textarea[placeholder*="OpenAPI document"]').setValue('openapi: 3.0.0\ninfo:\n  title: Orders')
    await flushPromises()

    expect(wrapper.text()).toContain('demo-org')
    expect(wrapper.text()).toContain('demo')
    expect(wrapper.text()).toContain('Ready to generate')
    expect(invokeMock).not.toHaveBeenCalledWith('generate_proxy_bundle', expect.anything())
    expect(invokeMock).not.toHaveBeenCalledWith('deploy_proxy', expect.anything())
  })

  it('reviews an existing not-deployed revision before any deploy command', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return [{ id: 'demo-org', project_id: 'demo-project', location: null }]
      if (command === 'list_environments') return [{ name: 'demo' }]
      if (command === 'list_templates') return []
      if (command === 'list_proxies') return [{ name: 'orders', revisions: [{ number: 2, deployed: false, status: 'NotDeployed' }] }]
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await wrapper.find('button[aria-label="Proxies"]').trigger('click')
    await vi.waitFor(() => expect(invokeMock).toHaveBeenCalledWith('list_proxies', { organization: 'demo-org', environment: 'demo' }))
    await vi.waitFor(() => expect(wrapper.find('.proxy-list__button').exists()).toBe(true))
    await wrapper.find('.proxy-list__button').trigger('click')
    await vi.waitFor(() => expect(wrapper.find('.revision-row__actions button').exists()).toBe(true))

    await wrapper.find('.revision-row__actions button').trigger('click')
    expect(wrapper.text()).toContain('Review proxy revision')
    expect(wrapper.text()).toContain('demo-org')
    expect(wrapper.text()).toContain('Confirm review')

    await wrapper.find('button.primary-action').trigger('click')
    await flushPromises()
    document.querySelector<HTMLButtonElement>('.base-modal__actions button:last-child')?.click()
    await flushPromises()
    expect(wrapper.text()).toContain('Review confirmed')
    expect(invokeMock).not.toHaveBeenCalledWith('deploy_proxy', expect.anything())
  })

  it('loads context and proxies only after explicit selections', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'auth_status') return { authenticated: false }
      if (command === 'auth_logout') return null
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
    expect(wrapper.find('.sidebar__avatar').text()).toBe('DE')
    expect(wrapper.find('.sidebar__profile-tooltip').text()).toContain('developer@example.com')
    expect(invokeMock).toHaveBeenCalledWith('list_environments', { organization: 'org-one' })
    expect(invokeMock).toHaveBeenCalledWith('list_proxies', { organization: 'org-one', environment: 'test' })

    await wrapper.find('.profile-tooltip__logout').trigger('click')
    await flushPromises()
    expect(invokeMock).toHaveBeenCalledWith('auth_logout', undefined)
    expect(wrapper.find('button.primary-action').exists()).toBe(true)
  })
})
