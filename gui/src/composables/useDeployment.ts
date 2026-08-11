import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { getCurrentInstance, onBeforeUnmount, ref } from 'vue'
import type { DeploymentDto } from '../types/bridge'
import type { Invoke } from './useAuth'

export type DeploymentTarget = {
  organization: string
  environment: string
  proxyName: string
  revision: number
}

export type DeploymentExecutionStatus = 'idle' | 'deploying' | 'polling' | 'succeeded' | 'failed' | 'stopped' | 'timeout' | 'error'

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)
const DEFAULT_POLL_INTERVAL_MS = 2000
const DEFAULT_POLL_TIMEOUT_MS = 5 * 60 * 1000

function errorMessage(caught: unknown, fallback: string): string {
  if (typeof caught === 'object' && caught !== null && 'message' in caught) {
    const message = (caught as { message?: unknown }).message
    if (typeof message === 'string' && message.trim()) return message
  }
  return fallback
}

function terminal(status: DeploymentDto['status']): boolean {
  return status === 'Succeeded' || status === 'Failed'
}

export function useDeployment(invoke: Invoke = defaultInvoke) {
  const result = ref<DeploymentDto | null>(null)
  const status = ref<DeploymentExecutionStatus>('idle')
  const error = ref<string | null>(null)
  const lastUpdated = ref<Date | null>(null)
  let timer: ReturnType<typeof setTimeout> | null = null
  let runToken = 0

  function stopTimer() {
    if (timer !== null) {
      clearTimeout(timer)
      timer = null
    }
  }

  function stopPolling() {
    runToken += 1
    stopTimer()
    if (status.value === 'polling') status.value = 'stopped'
  }

  async function readStatus(target: DeploymentTarget, token: number): Promise<boolean> {
    if (token !== runToken) return true
    try {
      const deployment = await invoke<DeploymentDto>('get_deployment_status', {
        organization: target.organization,
        environment: target.environment,
        proxyName: target.proxyName,
        revision: target.revision,
      })
      if (token !== runToken) return true
      result.value = deployment
      lastUpdated.value = new Date()
      if (terminal(deployment.status)) {
        status.value = deployment.status === 'Succeeded' ? 'succeeded' : 'failed'
        return true
      }
      return false
    } catch (caught) {
      if (token === runToken) {
        status.value = 'error'
        error.value = errorMessage(caught, 'The deployment status could not be loaded.')
      }
      return true
    }
  }

  async function startPolling(target: DeploymentTarget, intervalMs = DEFAULT_POLL_INTERVAL_MS, timeoutMs = DEFAULT_POLL_TIMEOUT_MS) {
    stopTimer()
    runToken += 1
    const token = runToken
    const deadline = Date.now() + timeoutMs
    status.value = 'polling'
    error.value = null

    const tick = async () => {
      if (token !== runToken) return
      const finished = await readStatus(target, token)
      if (finished || token !== runToken) return
      if (Date.now() >= deadline) {
        status.value = 'timeout'
        error.value = 'Deployment status polling timed out.'
        return
      }
      timer = setTimeout(() => { void tick() }, Math.max(250, intervalMs))
    }

    await tick()
  }

  async function deploy(target: DeploymentTarget, overrideExisting = false) {
    stopPolling()
    runToken += 1
    const token = runToken
    status.value = 'deploying'
    error.value = null
    result.value = null
    lastUpdated.value = null
    try {
      const deployment = await invoke<DeploymentDto>('deploy_proxy', {
        organization: target.organization,
        environment: target.environment,
        proxyName: target.proxyName,
        revision: target.revision,
        overrideExisting,
      })
      if (token !== runToken) return false
      result.value = deployment
      lastUpdated.value = new Date()
      if (terminal(deployment.status)) {
        status.value = deployment.status === 'Succeeded' ? 'succeeded' : 'failed'
      } else {
        await startPolling(target)
      }
      return true
    } catch (caught) {
      if (token === runToken) {
        status.value = 'error'
        error.value = errorMessage(caught, 'The proxy revision could not be deployed.')
      }
      return false
    }
  }

  async function retry(target: DeploymentTarget) {
    return deploy(target, false)
  }

  function reset() {
    runToken += 1
    stopTimer()
    result.value = null
    status.value = 'idle'
    error.value = null
    lastUpdated.value = null
  }

  if (getCurrentInstance()) onBeforeUnmount(stopPolling)

  return { result, status, error, lastUpdated, deploy, startPolling, stopPolling, retry, reset }
}
