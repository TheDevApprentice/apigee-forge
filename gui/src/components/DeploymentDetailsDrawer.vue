<script setup lang="ts">
import type { DeploymentDto, ProxyDto, ProxyRevisionDto } from '../types/bridge'
import BaseChip from './base/BaseChip.vue'
import BaseDrawer from './base/BaseDrawer.vue'

defineProps<{
  open: boolean
  proxy: ProxyDto
  revision: ProxyRevisionDto
  organization: string
  environment: string
  demo: boolean
  confirmed: boolean
  deployment: DeploymentDto | null
  status: string
  lastUpdated: Date | null
  error: string | null
}>()

const emit = defineEmits<{
  close: []
  confirm: []
  deploy: []
  stop: []
  retry: []
}>()
</script>

<template>
  <BaseDrawer :open="open" eyebrow="Deployment review" :title="`${proxy.name} · Revision ${revision.number}`" close-label="Close deployment details" @close="emit('close')">
    <div class="deployment-review">
      <div class="deployment-preparation__header">
        <div>
          <p>Confirm the existing revision and target before deployment.</p>
        </div>
        <BaseChip :label="confirmed ? 'Review confirmed' : 'Confirmation required'" :tone="confirmed ? 'success' : 'warning'" />
      </div>
      <dl class="review-grid">
        <div><span>Mode</span><strong>{{ demo ? 'Demo' : 'Live' }}</strong></div>
        <div><span>Organization</span><strong>{{ organization }}</strong></div>
        <div><span>Environment</span><strong>{{ environment }}</strong></div>
        <div><span>Proxy</span><strong>{{ proxy.name }}</strong></div>
        <div><span>Revision</span><strong>{{ revision.number }}</strong></div>
        <div><span>Current status</span><strong>{{ deployment?.status || revision.status }}</strong></div>
      </dl>
      <div v-if="deployment" class="deployment-preparation__created" role="status" aria-live="polite">
        Deployment status: <strong>{{ deployment.status }}</strong>
        <span v-if="lastUpdated"> · updated {{ lastUpdated.toLocaleTimeString() }}</span>
      </div>
      <p v-if="status === 'polling'" class="deployment-preparation__next-step" role="status" aria-live="polite">Apigee is still processing the revision. The GUI will continue polling for up to five minutes.</p>
      <p v-if="error" class="deployment-preparation__warning" role="alert">{{ error }}</p>
      <div class="review-actions">
        <button v-if="!confirmed" type="button" class="primary-action" @click="emit('confirm')">Confirm review</button>
        <button v-else type="button" class="primary-action" :disabled="status === 'deploying' || status === 'polling' || status === 'succeeded'" @click="emit('deploy')">
          {{ status === 'deploying' ? 'Deploying…' : status === 'polling' ? 'Waiting for status…' : status === 'succeeded' ? 'Deployment succeeded' : 'Deploy revision' }}
        </button>
        <button v-if="status === 'polling'" type="button" @click="emit('stop')">Stop polling</button>
        <button v-else-if="['failed', 'error', 'timeout', 'stopped'].includes(status)" type="button" @click="emit('retry')">Retry deployment</button>
      </div>
    </div>
  </BaseDrawer>
</template>
