import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import App from '../App.vue'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

describe('App M6-Bis flow', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  afterEach(() => {
    document.body.innerHTML = ''
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
    expect(wrapper.find('.sidebar').exists()).toBe(false)
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

  it('completes the Demo creation-to-deployment journey', async () => {
    let proxyListCalls = 0
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
      if (command === 'list_proxies') {
        proxyListCalls += 1
        return [{ name: 'api-orders', revisions: [{ number: 2, deployed: proxyListCalls > 2, status: proxyListCalls > 2 ? 'Succeeded' : 'NotDeployed' }] }]
      }
      if (command === 'generate_proxy_bundle') return { job_id: 'demo-job', proxy_name: 'api-orders', rendered_file_count: 7, state: 'Ready' }
      if (command === 'upload_proxy_bundle') return { source: 'demo', organization: 'demo-org', proxy_name: 'api-orders', revision: 2, deployed: false }
      if (command === 'deploy_proxy') return { source: 'demo', id: 'demo-deployment', organization: 'demo-org', environment: 'demo', proxy_name: 'api-orders', revision: 2, status: 'Pending' }
      if (command === 'get_deployment_status') return { source: 'demo', id: 'demo-deployment', organization: 'demo-org', environment: 'demo', proxy_name: 'api-orders', revision: 2, status: 'Succeeded' }
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await wrapper.findAll('.dashboard-action-card')[1].trigger('click')
    await vi.waitFor(() => expect(wrapper.find('select[aria-label="Select proxy template"]').exists()).toBe(true))
    await wrapper.find('select[aria-label="Select proxy template"]').setValue('orders')
    await wrapper.findAll('.workspace-selectors select')[0].setValue('demo-org')
    await vi.waitFor(() => expect(wrapper.find('.workspace-selectors select option[value="demo"]').exists()).toBe(true))
    await wrapper.findAll('.workspace-selectors select')[1].setValue('demo')
    await flushPromises()
    await vi.waitFor(() => expect(wrapper.text()).toContain('Organizationdemo-org'))
    await wrapper.find('input[placeholder="openapi.yaml"]').setValue('orders.yaml')
    await wrapper.find('textarea[placeholder*="OpenAPI document"]').setValue('openapi: 3.0.0\nservers:\n  - url: https://api.example.test')
    await flushPromises()

    await wrapper.find('button.primary-action').trigger('click')
    await vi.waitFor(() => expect(wrapper.text()).toContain('Upload and create proxy'))
    await wrapper.findAll('button').find((button) => button.text() === 'Upload and create proxy')?.trigger('click')
    await vi.waitFor(() => expect(proxyListCalls).toBeGreaterThan(1))

    await wrapper.findAll('button').find((button) => button.text() === 'Back to templates')?.trigger('click')
    await vi.waitFor(() => expect(wrapper.find('.proxy-list__button').exists()).toBe(true))
    await wrapper.find('.proxy-list__button').trigger('click')
    document.querySelector<HTMLButtonElement>('.revision-row__actions button')?.click()
    await nextTick()
    await wrapper.find('button.primary-action').trigger('click')
    document.querySelector<HTMLButtonElement>('.base-modal__actions button:last-child')?.click()
    await flushPromises()
    await wrapper.findAll('button').find((button) => button.text() === 'Deploy revision')?.trigger('click')
    await vi.waitFor(() => expect(wrapper.text()).toContain('Deployment succeeded'))
    expect(invokeMock).toHaveBeenCalledWith('get_deployment_status', {
      organization: 'demo-org',
      environment: 'demo',
      proxyName: 'api-orders',
      revision: 2,
    })
  })

  it('opens selected proxy details in a right drawer and closes it', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'session_status') return { mode: 'demo', status: 'ready', identity: null, organization: 'demo-org', environment: 'demo', error: null }
      if (command === 'list_organizations') return [{ id: 'demo-org', project_id: 'demo-project', location: null }]
      if (command === 'list_environments') return [{ name: 'demo' }]
      if (command === 'list_templates') return []
      if (command === 'list_proxies') return [{ name: 'orders', revisions: [{ number: 1, deployed: false, status: 'NotDeployed' }] }]
      throw new Error(`Unexpected command ${command}`)
    })

    const wrapper = mount(App)
    await flushPromises()
    await wrapper.find('button[aria-label="Proxies"]').trigger('click')
    await vi.waitFor(() => expect(wrapper.find('.proxy-list__button').exists()).toBe(true))
    await wrapper.find('.proxy-list__button').trigger('click')
    expect(document.querySelector('.proxy-drawer')).not.toBeNull()
    expect(wrapper.text()).toContain('orders')

    document.querySelector<HTMLButtonElement>('.proxy-drawer__close')?.click()
    await nextTick()
    await flushPromises()
    expect(document.querySelector('.proxy-drawer')).toBeNull()
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
    await vi.waitFor(() => expect(document.querySelector('.revision-row__actions button')).not.toBeNull())

    document.querySelector<HTMLButtonElement>('.revision-row__actions button')?.click()
    await nextTick()
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
