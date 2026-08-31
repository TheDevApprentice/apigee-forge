<script setup lang="ts">
import BaseCard from './BaseCard.vue'
import BaseEmptyState from './BaseEmptyState.vue'
import BaseErrorState from './BaseErrorState.vue'
import BaseSpinner from './BaseSpinner.vue'

export type CollectionFilter = {
  value: string
  label: string
  count?: number
}

withDefaults(defineProps<{
  eyebrow?: string
  sectionLabel?: string
  title: string
  description?: string
  searchable?: boolean
  searchValue?: string
  searchPlaceholder?: string
  filters?: CollectionFilter[]
  activeFilter?: string
  loading?: boolean
  error?: string | null
  empty?: boolean
  emptyTitle?: string
  emptyHint?: string
}>(), {
  description: '',
  searchable: true,
  searchValue: '',
  searchPlaceholder: 'Search',
  filters: () => [],
  activeFilter: 'all',
  loading: false,
  error: null,
  empty: false,
  emptyTitle: 'Nothing to show',
  emptyHint: 'There are no items to display yet.',
})

const emit = defineEmits<{
  'update:searchValue': [value: string]
  'update:activeFilter': [value: string]
  retry: []
}>()
</script>

<template>
  <BaseCard class="collection-list" :eyebrow="eyebrow">
    <header class="collection-list__header">
      <div>
        <p v-if="sectionLabel" class="collection-list__section-label">{{ sectionLabel }}</p>
        <h2 class="collection-list__label">{{ title }}</h2>
        <p v-if="description" class="collection-list__description">{{ description }}</p>
      </div>
      <div class="collection-list__actions"><slot name="actions" /></div>
    </header>
    <div v-if="searchable || filters.length" class="collection-list__toolbar">
      <label v-if="searchable" class="collection-list__search-wrap">
        <span class="sr-only">{{ searchPlaceholder }}</span>
        <span class="collection-list__search-icon" aria-hidden="true">⌕</span>
        <input class="collection-list__search proxy-search" type="search" :value="searchValue" :placeholder="searchPlaceholder" :aria-label="searchPlaceholder" @input="emit('update:searchValue', ($event.target as HTMLInputElement).value)" />
      </label>
      <nav v-if="filters.length" class="collection-list__filters" aria-label="List filters">
        <button v-for="filter in filters" :key="filter.value" type="button" class="collection-list__filter" :class="{ 'collection-list__filter--active': activeFilter === filter.value }" :aria-pressed="activeFilter === filter.value" @click="emit('update:activeFilter', filter.value)">
          <span>{{ filter.label }}</span><small v-if="filter.count !== undefined">{{ filter.count }}</small>
        </button>
      </nav>
    </div>
    <div v-if="loading" class="collection-list__state" aria-live="polite"><BaseSpinner /><span>Loading {{ title.toLowerCase() }}…</span></div>
    <BaseErrorState v-else-if="error" @retry="emit('retry')">
      <template #title>Could not load {{ title.toLowerCase() }}</template>
      <template #hint>{{ error }}</template>
    </BaseErrorState>
    <slot v-else-if="!empty" />
    <slot v-else name="empty"><BaseEmptyState><template #title>{{ emptyTitle }}</template><template #hint>{{ emptyHint }}</template></BaseEmptyState></slot>
  </BaseCard>
</template>
