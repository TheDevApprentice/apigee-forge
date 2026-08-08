import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { ProxyDto } from '../types/bridge'
import type { Invoke } from './useAuth'

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)

export function useProxies(invoke: Invoke = defaultInvoke) {
  const proxies = ref<ProxyDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(organization: string, environment: string) {
    loading.value = true
    error.value = null
    try {
      proxies.value = await invoke<ProxyDto[]>('list_proxies', { organization, environment })
    } catch {
      error.value = 'Proxies could not be loaded.'
    } finally {
      loading.value = false
    }
  }

  return { proxies, loading, error, load }
}
