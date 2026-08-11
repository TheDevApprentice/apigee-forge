<script setup lang="ts">
import BaseChip from './base/BaseChip.vue'
import type { OpenApiSourceDto, ProxyCreationPreviewDto } from '../types/bridge'
import type { ProxyCreationPreparationErrors } from '../composables/useDeploymentPreparation'

defineProps<{
  openApiSource: OpenApiSourceDto
  proxyName: string
  errors: ProxyCreationPreparationErrors
  ready: boolean
  preview: ProxyCreationPreviewDto | null
  logicalTargetEnvironment: string | null
}>()

defineEmits<{
  updateOpenApiDisplayName: [value: string]
  updateOpenApiContent: [value: string]
  updateProxyName: [value: string]
  cancel: []
}>()
</script>

<template>
  <div class="deployment-preparation">
    <div class="deployment-preparation__header">
      <div>
        <h2>Prepare proxy creation</h2>
        <p>Review the local inputs before generating and uploading a proxy revision to Apigee.</p>
      </div>
      <BaseChip :label="ready ? 'Ready to generate' : 'Needs attention'" />
    </div>
    <div class="deployment-preparation__form">
      <label>
        <span>OpenAPI display name</span>
        <input :value="openApiSource.display_name" type="text" placeholder="openapi.yaml" @input="$emit('updateOpenApiDisplayName', ($event.target as HTMLInputElement).value)" />
        <small v-if="errors.spec" class="deployment-preparation__field-error">{{ errors.spec }}</small>
      </label>
      <label>
        <span>OpenAPI document</span>
        <textarea :value="openApiSource.content" rows="8" placeholder="Paste an OpenAPI document for this local preview" @input="$emit('updateOpenApiContent', ($event.target as HTMLTextAreaElement).value)" />
      </label>
      <label>
        <span>Proxy name</span>
        <input :value="proxyName" type="text" @input="$emit('updateProxyName', ($event.target as HTMLInputElement).value)" />
        <small v-if="errors.proxy" class="deployment-preparation__field-error">{{ errors.proxy }}</small>
      </label>
    </div>
    <div class="deployment-preparation__preview" role="status" aria-live="polite">
      <div class="deployment-preparation__preview-header"><strong>Proxy creation preview</strong><span>Non-mutating</span></div>
      <dl v-if="preview">
        <div><dt>Template</dt><dd>{{ preview.template_name }}</dd></div>
        <div><dt>OpenAPI</dt><dd>{{ preview.spec_display_name || 'Not provided' }}</dd></div>
        <div><dt>Organization</dt><dd>{{ preview.organization || 'Not selected' }}</dd></div>
        <div><dt>Apigee environment</dt><dd>{{ preview.environment || 'Not selected' }}</dd></div>
        <div><dt>Proxy</dt><dd>{{ preview.proxy_name || 'Not resolved' }}</dd></div>
        <div><dt>Policies</dt><dd>{{ preview.policy_count }}</dd></div>
      </dl>
      <p v-if="logicalTargetEnvironment && preview?.logical_target_matches === false" class="deployment-preparation__warning">
        Template target “{{ logicalTargetEnvironment }}” is a logical target. It differs from the selected Apigee environment; confirm the mapping explicitly before uploading.
      </p>
    </div>
    <div v-if="Object.keys(errors).length" class="deployment-preparation__errors" role="alert" aria-live="assertive">
      <strong>Proxy creation cannot continue</strong>
      <span v-for="(message, field) in errors" :key="field">{{ message }}</span>
    </div>
    <div class="review-actions">
      <button type="button" @click="$emit('cancel')">Back to templates</button>
      <button type="button" class="primary-action" disabled>Continue to bundle generation</button>
    </div>
    <p class="deployment-preparation__next-step">Bundle generation will be connected in M8-02. This screen does not write files or call Apigee.</p>
  </div>
</template>
