import { describe, expect, it, vi } from 'vitest'
import { useProxyCreationPreparation, deriveProxyName } from '../composables/useDeploymentPreparation'
import type { Invoke } from '../composables/useAuth'
import type { TemplateDto } from '../types/bridge'

const template: TemplateDto = {
  name: 'orders',
  data: {
    metadata: {
      name: 'orders',
      owner: 'platform',
      target_environment: 'prod',
      naming_convention: { prefix: 'api-', case: 'kebab-case' },
    },
    flow: {
      pre_flow: { request: [{ type: 'cors' }], response: [] },
      conditional_flows: [],
      post_flow: { request: [], response: [] },
    },
  },
}

const openApi = `openapi: 3.0.0\ninfo:\n  title: Orders\n  version: 1.0.0`

describe('useProxyCreationPreparation', () => {
  it('derives a proxy name from the template naming convention', () => {
    expect(deriveProxyName(template)).toBe('api-orders')
  })

  it('builds a non-mutating preview only when all inputs are valid', () => {
    const preparation = useProxyCreationPreparation()
    preparation.selectTemplate(template)
    preparation.setOpenApiSource({ display_name: 'orders.yaml', content: openApi })
    preparation.setContext('apigee-forge', 'eval')

    expect(preparation.ready.value).toBe(true)
    expect(preparation.preview.value).toEqual({
      template_name: 'orders',
      spec_display_name: 'orders.yaml',
      organization: 'apigee-forge',
      environment: 'eval',
      proxy_name: 'api-orders',
      logical_target_environment: 'prod',
      logical_target_matches: false,
      policy_count: 1,
    })
    expect(preparation.jobInput()?.proxy_name).toBe('api-orders')
  })

  it('generates locally and uploads the generated job without deploying it', async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === 'generate_proxy_bundle') return { job_id: 'gui-job-1', proxy_name: 'api-orders', rendered_file_count: 9, state: 'Ready' }
      if (command === 'upload_proxy_bundle') return { source: 'demo', organization: 'demo-org', proxy_name: 'api-orders', revision: 2, deployed: false }
      throw new Error(`Unexpected command ${command}`)
    }) as unknown as Invoke
    const preparation = useProxyCreationPreparation(invoke)
    preparation.selectTemplate(template)
    preparation.setOpenApiSource({ display_name: 'orders.yaml', content: openApi })
    preparation.setContext('demo-org', 'demo')

    expect(await preparation.generate()).toBe(true)
    expect(preparation.generation.value?.job_id).toBe('gui-job-1')
    expect(await preparation.upload()).toBe(true)
    expect(preparation.createdRevision.value).toMatchObject({ revision: 2, deployed: false })
    expect(invoke).toHaveBeenCalledWith('generate_proxy_bundle', {
      template: template.data,
      openapiSource: openApi,
      proxyName: 'api-orders',
    })
    expect(invoke).toHaveBeenCalledWith('upload_proxy_bundle', {
      organization: 'demo-org',
      proxyName: 'api-orders',
      jobId: 'gui-job-1',
    })
  })

  it('shows the safe Rust error when local generation fails', async () => {
    const invoke = vi.fn().mockRejectedValue({ code: 'TEMPLATE_INVALID', message: 'The selected template is invalid for bundle generation' }) as unknown as Invoke
    const preparation = useProxyCreationPreparation(invoke)
    preparation.selectTemplate(template)
    preparation.setOpenApiSource({ display_name: 'orders.yaml', content: openApi })
    preparation.setContext('demo-org', 'demo')

    expect(await preparation.generate()).toBe(false)
    expect(preparation.error.value).toBe('The selected template is invalid for bundle generation')
  })

  it('rejects incomplete input and distinguishes the logical target from Apigee environment', () => {
    const preparation = useProxyCreationPreparation()
    preparation.selectTemplate(template)
    preparation.setOpenApiSource({ display_name: 'orders.yaml', content: 'not an OpenAPI document' })
    preparation.setContext('', '')

    expect(preparation.ready.value).toBe(false)
    expect(preparation.errors.value).toMatchObject({
      spec: 'The specification must contain a valid OpenAPI document.',
      organization: 'Select an Apigee organization.',
      environment: 'Select an Apigee environment.',
    })
    expect(preparation.preview.value?.logical_target_environment).toBe('prod')
    expect(preparation.jobInput()).toBeNull()
  })
})
