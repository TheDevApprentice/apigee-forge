<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ref } from 'vue'
import BaseSelect from '../base/BaseSelect.vue'

type SelectOption = { value: string; label: string; description?: string }
type AppMode = 'demo' | 'cloud'
type SearchResult = { id: string; title: string; description: string; target: string }
type SearchGroup = { category: string; results: SearchResult[] }

withDefaults(defineProps<{
  identity?: string
  demo?: boolean
  authenticated?: boolean
  mode: AppMode | null
  organization: string
  environment: string
  organizations: SelectOption[]
  environments: SelectOption[]
  loading?: boolean
  searchQuery?: string
  searchResults?: SearchGroup[]
}>(), {
  identity: '', demo: false, authenticated: false, mode: 'cloud', organization: '', environment: '', organizations: () => [], environments: () => [], loading: false, searchQuery: '', searchResults: () => [],
})

const emit = defineEmits<{
  'update:mode': [value: AppMode]
  'update:organization': [value: string]
  'update:environment': [value: string]
  'update:searchQuery': [value: string]
  navigate: [target: string]
}>()

const searchFocused = ref(false)

async function minimize() { await getCurrentWindow().minimize() }
async function toggleMaximize() { await getCurrentWindow().toggleMaximize() }
async function close() { await getCurrentWindow().close() }
function clearSearch() { emit('update:searchQuery', '') }
function selectResult(target: string) { emit('navigate', target); clearSearch() }
</script>

<template>
  <header class="desktop-titlebar" data-tauri-drag-region aria-label="Application title bar">
    <div class="desktop-titlebar__left" data-tauri-drag-region>
      <div class="desktop-titlebar__brand" data-tauri-drag-region>
        <span class="desktop-titlebar__mark" aria-hidden="true">AF</span>
        <span>Apigee Forge</span>
      </div>
      <div v-if="authenticated" class="desktop-titlebar__workspace">
        <span class="desktop-titlebar__workspace-label">Workspace</span>
        <BaseSelect :model-value="organization" label="Organization" placeholder="Organization" :disabled="loading" :options="organizations" @update:model-value="emit('update:organization', $event)" />
        <span class="desktop-titlebar__separator" aria-hidden="true">/</span>
        <BaseSelect v-if="organization" :model-value="environment" label="Environment" placeholder="Environment" :disabled="loading || !environments.length" :options="environments" @update:model-value="emit('update:environment', $event)" />
        <span v-else class="desktop-titlebar__placeholder">Choose organization</span>
      </div>
    </div>
    <div class="desktop-titlebar__search" :class="{ 'desktop-titlebar__search--focused': searchFocused }">
      <span class="sr-only">Search workspace</span>
      <span class="desktop-titlebar__search-icon" aria-hidden="true"></span>
      <input :value="searchQuery" type="search" :placeholder="authenticated ? 'Search workspace' : 'Search support'" @focus="searchFocused = true" @blur="searchFocused = false" @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)" @keydown.esc="clearSearch" />
      <button v-if="searchQuery" type="button" class="desktop-titlebar__search-clear" aria-label="Clear search" title="Clear search" @mousedown.prevent @click="clearSearch">×</button>
      <kbd v-else>⌘ K</kbd>
      <div v-if="searchFocused && searchQuery.trim()" class="desktop-titlebar__results" role="dialog" aria-label="Search results">
        <div v-if="searchResults.length" class="desktop-titlebar__result-groups">
          <section v-for="group in searchResults" :key="group.category" class="desktop-titlebar__result-group"><p>{{ group.category }}</p><button v-for="result in group.results" :key="result.id" type="button" @mousedown.prevent @click="selectResult(result.target)"><span class="desktop-titlebar__result-icon" aria-hidden="true">→</span><span><strong>{{ result.title }}</strong><small>{{ result.description }}</small></span></button></section>
        </div>
        <p v-else class="desktop-titlebar__no-results">No results found. Try another search.</p>
      </div>
    </div>
    <div class="desktop-titlebar__actions">
      <span class="desktop-titlebar__identity" :title="identity || (demo ? 'Demo workspace' : 'Workspace')">{{ identity || (demo ? 'Demo workspace' : 'Workspace') }}</span>
      <label class="desktop-titlebar__mode mode-switcher"><span class="sr-only">Workspace mode</span><select :value="mode || 'cloud'" @change="emit('update:mode', ($event.target as HTMLSelectElement).value as AppMode)"><option value="cloud">Live</option><option value="demo">Demo</option></select></label>
      <button type="button" aria-label="Minimize window" title="Minimize" @click="minimize">−</button>
      <button type="button" aria-label="Maximize window" title="Maximize" @click="toggleMaximize">□</button>
      <button type="button" class="desktop-titlebar__close" aria-label="Close window" title="Close" @click="close">×</button>
    </div>
  </header>
</template>
