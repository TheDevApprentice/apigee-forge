import { ref } from 'vue'
import type { AppMode, SessionDto } from '../types/bridge'

export function useSession(initial: SessionDto | null = null) {
  const session = ref<SessionDto | null>(initial)
  const selectedMode = ref<AppMode | null>(initial?.mode ?? null)

  function apply(next: SessionDto) {
    session.value = next
    selectedMode.value = next.mode
  }

  function clear() {
    session.value = null
    selectedMode.value = null
  }

  return { session, selectedMode, apply, clear }
}
