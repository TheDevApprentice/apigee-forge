import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { ProxyDto } from '../types/bridge'

export function useProxies() {
  const proxies = ref<ProxyDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(organization: string) {
    loading.value = true
    error.value = null
    try {
      proxies.value = await invoke<ProxyDto[]>('list_proxies', { organization })
    } catch {
      error.value = 'Proxies could not be loaded.'
    } finally {
      loading.value = false
    }
  }

  return { proxies, loading, error, load }
}
