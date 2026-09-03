<script setup lang="ts">
import type { ProxyDto, RevisionDetailDto } from '../types/bridge'
import BaseChip from './base/BaseChip.vue'
import BaseDrawer from './base/BaseDrawer.vue'
import BaseErrorState from './base/BaseErrorState.vue'
import BaseSpinner from './base/BaseSpinner.vue'

defineProps<{
  open: boolean
  proxy: ProxyDto | null
  organization: string
  environment: string
  selectedRevision: number | null
  revisionDetail: RevisionDetailDto | null
  revisionDetailLoading: boolean
  revisionDetailError: string | null
}>()

const emit = defineEmits<{
  close: []
  toggleRevision: [revision: number]
  retryRevision: [revision: number]
  reviewDeployment: [revision: ProxyDto['revisions'][number]]
}>()
</script>

<template>
  <BaseDrawer v-if="proxy" :open="open" eyebrow="Selected proxy details" :title="proxy.name" close-label="Close proxy details" @close="emit('close')">
    <div class="proxy-detail">
      <div class="proxy-detail__header">
        <p>{{ proxy.source === 'cloud' ? 'Live Apigee proxy' : 'Demo proxy' }}</p>
        <BaseChip :label="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'Deployed' : 'Not deployed'" :tone="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'success' : 'neutral'" />
      </div>
      <dl class="proxy-metadata">
        <div><dt>Organization</dt><dd>{{ organization }}</dd></div>
        <div><dt>Environment</dt><dd>{{ environment }}</dd></div>
        <div><dt>Revision count</dt><dd>{{ proxy.revisions.length }}</dd></div>
      </dl>
      <h3>Revisions</h3>
      <ul class="proxy-revisions">
        <li v-for="revision in proxy.revisions" :key="revision.number">
          <button type="button" class="revision-row__button" @click="emit('toggleRevision', revision.number)">
            <span>Revision {{ revision.number }}</span>
            <BaseChip :label="revision.status === 'Succeeded' ? 'Deployed' : revision.status === 'NotDeployed' ? 'Not deployed' : revision.status" :tone="revision.status === 'Succeeded' ? 'success' : revision.status === 'NotDeployed' ? 'neutral' : 'warning'" />
          </button>
          <div v-if="selectedRevision === revision.number" class="revision-detail">
            <BaseSpinner v-if="revisionDetailLoading" />
            <BaseErrorState v-else-if="revisionDetailError" @retry="emit('retryRevision', revision.number)">
              <template #title>Revision unavailable</template>
              <template #hint>{{ revisionDetailError }}</template>
            </BaseErrorState>
            <dl v-else-if="revisionDetail" class="revision-detail__metadata">
              <div><dt>Revision</dt><dd>{{ revisionDetail.revision }}</dd></div>
              <div><dt>Proxy</dt><dd>{{ revisionDetail.proxy_name }}</dd></div>
              <div><dt>API fields</dt><dd>{{ Object.keys(revisionDetail.data).length }}</dd></div>
            </dl>
          </div>
          <div class="revision-row__actions">
            <button v-if="revision.status === 'NotDeployed'" type="button" @click="emit('reviewDeployment', revision)">Review deployment</button>
            <span v-else-if="revision.status === 'Succeeded'" class="revision-row__hint">Already deployed</span>
            <span v-else class="revision-row__hint">{{ revision.status }}</span>
          </div>
        </li>
      </ul>
    </div>
  </BaseDrawer>
</template>
