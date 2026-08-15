<script setup lang="ts">
import { computed } from 'vue'
import type { TemplateDto } from '../types/bridge'
import BaseDrawer from './base/BaseDrawer.vue'

type TemplateMetadata = {
  description?: unknown
  owner?: unknown
  target_environment?: unknown
  naming_convention?: { prefix?: unknown; case?: unknown }
}

const emit = defineEmits<{
  close: []
  edit: []
  prepareProxy: []
  delete: []
}>()

const props = defineProps<{
  open: boolean
  template: TemplateDto | null
  deletePending: boolean
}>()

const metadata = computed<TemplateMetadata>(() => (props.template?.data.metadata as TemplateMetadata | undefined) || {})
const flow = computed<Record<string, any>>(() => (props.template?.data.flow as Record<string, any> | undefined) || {})
const policyCount = computed(() => {
  const stages = [flow.value.pre_flow, flow.value.post_flow, ...(Array.isArray(flow.value.conditional_flows) ? flow.value.conditional_flows : [])]
  return stages.reduce((total, stage) => total + (Array.isArray(stage?.request) ? stage.request.length : 0) + (Array.isArray(stage?.response) ? stage.response.length : 0), 0)
})
</script>

<template>
  <BaseDrawer v-if="template" :open="open" eyebrow="Template details" :title="template.name || 'Untitled template'" close-label="Close template details" @close="emit('close')">
    <div class="template-detail">
      <p class="template-detail__description">{{ metadata.description || 'No description provided.' }}</p>
      <dl class="template-metadata">
        <div><dt>Owner</dt><dd>{{ metadata.owner || 'No owner' }}</dd></div>
        <div><dt>Target environment</dt><dd>{{ metadata.target_environment || 'Not specified' }}</dd></div>
        <div><dt>Policies</dt><dd>{{ policyCount }}</dd></div>
        <div><dt>Name convention</dt><dd>{{ metadata.naming_convention?.prefix || 'No prefix' }} · {{ metadata.naming_convention?.case || 'kebab-case' }}</dd></div>
      </dl>
      <div class="template-drawer__actions">
        <button type="button" class="primary-action" @click="emit('prepareProxy')">Create proxy from template</button>
        <button type="button" class="template-drawer__edit" @click="emit('edit')">Edit template</button>
        <button type="button" class="template-drawer__delete" :disabled="deletePending" @click="emit('delete')">{{ deletePending ? 'Deleting…' : 'Delete template' }}</button>
      </div>
    </div>
  </BaseDrawer>
</template>
