import { afterEach, describe, expect, it, vi } from 'vitest'
import { useDeployment, type DeploymentTarget } from '../composables/useDeployment'
import type { Invoke } from '../composables/useAuth'

const target: DeploymentTarget = {
  organization: 'demo-org',
  environment: 'demo',
  proxyName: 'api-orders',
  revision: 2,
}

afterEach(() => {
  vi.useRealTimers()
})

describe('useDeployment', () => {
  it('deploys a confirmed revision with explicit camelCase IPC arguments', async () => {
    const invoke = vi.fn().mockResolvedValue({
      source: 'demo',
      id: 'deployment-2',
      organization: 'demo-org',
      environment: 'demo',
      proxy_name: 'api-orders',
      revision: 2,
      status: 'Succeeded',
    }) as unknown as Invoke
    const deployment = useDeployment(invoke)

    expect(await deployment.deploy(target)).toBe(true)
    expect(deployment.status.value).toBe('succeeded')
    expect(deployment.result.value?.revision).toBe(2)
    expect(invoke).toHaveBeenCalledWith('deploy_proxy', {
      organization: 'demo-org',
      environment: 'demo',
      proxyName: 'api-orders',
      revision: 2,
      overrideExisting: false,
    })
  })

  it('polls until a pending deployment reaches a terminal status and can be stopped', async () => {
    vi.useFakeTimers()
    const statuses = ['InProgress', 'Succeeded']
    const invoke = vi.fn(async (command: string) => {
      if (command === 'deploy_proxy') return { source: 'demo', id: 'deployment-2', ...target, proxy_name: target.proxyName, status: 'Pending' }
      const status = statuses.shift() || 'Succeeded'
      return { source: 'demo', id: 'deployment-2', ...target, proxy_name: target.proxyName, status }
    }) as unknown as Invoke
    const deployment = useDeployment(invoke)

    await deployment.deploy(target)
    expect(deployment.status.value).toBe('polling')
    await vi.advanceTimersByTimeAsync(1000)
    expect(deployment.status.value).toBe('succeeded')
    expect(invoke).toHaveBeenCalledWith('get_deployment_status', {
      organization: 'demo-org',
      environment: 'demo',
      proxyName: 'api-orders',
      revision: 2,
    })
  })
})
