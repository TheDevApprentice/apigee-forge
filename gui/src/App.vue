<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useAuth } from './composables/useAuth'
import BaseButton from './components/base/BaseButton.vue'
import BaseCard from './components/base/BaseCard.vue'
import BaseChip from './components/base/BaseChip.vue'
import BaseEmptyState from './components/base/BaseEmptyState.vue'

type NavigationItem = {
  label: string
  path: string
}

const navigation: NavigationItem[] = [
  { label: 'Dashboard', path: 'M3 12 12 3l9 9M5 10v10h14V10' },
  { label: 'Templates', path: 'M4 5h16v14H4zM8 9h8M8 13h5' },
  { label: 'Proxies', path: 'M5 7h14M5 12h14M5 17h14' },
  { label: 'Deployments', path: 'M12 3v13M7 11l5 5 5-5M5 21h14' },
  { label: 'Settings', path: 'M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1' },
]

const activeView = ref('Dashboard')
const auth = useAuth()

onMounted(() => {
  void auth.refresh()
})
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar" aria-label="Primary navigation">
      <div class="brand-mark" aria-label="Apigee Forge" title="Apigee Forge">AF</div>
      <nav class="sidebar__nav">
        <BaseButton
          v-for="item in navigation"
          :key="item.label"
          :label="item.label"
          :active="activeView === item.label"
          @click="activeView = item.label"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path :d="item.path" />
          </svg>
        </BaseButton>
      </nav>
      <div class="sidebar__footer">
        <span class="connection-dot" aria-label="Offline workspace" title="Offline workspace" />
      </div>
    </aside>

    <div class="app-frame">
      <header class="topbar">
        <div>
          <p class="topbar__eyebrow">Workspace</p>
          <p class="topbar__context">No organization selected <span>/</span> No environment selected</p>
        </div>
        <BaseChip label="M6 preview" />
      </header>

      <main class="main-content">
        <div class="page-heading">
          <div>
            <p class="page-heading__eyebrow">{{ activeView }}</p>
            <h1>Apigee Forge</h1>
          </div>
          <span class="page-heading__status">GUI shell ready</span>
        </div>

        <BaseCard eyebrow="M6 · Foundation">
          <div class="welcome-card">
            <div class="welcome-card__copy">
              <h2>A calm workspace for API delivery.</h2>
              <p>The Tauri and Vue shell is running. Live data will appear here after the typed command bridge is connected.</p>
            </div>
            <div class="welcome-card__signal" aria-hidden="true">
              <span />
              <span />
              <span />
            </div>
          </div>
        </BaseCard>

        <BaseCard :eyebrow="`${activeView} data`">
          <BaseEmptyState>
            <template #title>No live data loaded</template>
            <template #hint>M6-02 will connect this surface to the core application.</template>
          </BaseEmptyState>
        </BaseCard>
      </main>
    </div>
  </div>
</template>
