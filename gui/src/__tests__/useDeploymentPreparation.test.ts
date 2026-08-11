import { describe, expect, it } from 'vitest'
import { useDeploymentPreparation, deriveProxyName } from '../composables/useDeploymentPreparation'
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

describe('useDeploymentPreparation', () => {
  it('derives a proxy name from the template naming convention', () => {
    expect(deriveProxyName(template)).toBe('api-orders')
  })

  it('builds a non-mutating preview only when all inputs are valid', () => {
    const preparation = useDeploymentPreparation()
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
    expect(preparation.jobInput()?.override_existing).toBe(false)
  })

  it('rejects incomplete input and distinguishes the logical target from Apigee environment', () => {
    const preparation = useDeploymentPreparation()
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
