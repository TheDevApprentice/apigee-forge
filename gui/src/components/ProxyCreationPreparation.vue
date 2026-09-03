<script setup lang="ts">
import { ref } from 'vue'
import BaseChip from './base/BaseChip.vue'
import type { BundleGenerationResultDto, CreatedProxyRevisionDto, OpenApiSourceDto, ProxyCreationPreviewDto, TemplateDto } from '../types/bridge'
import type { ProxyCreationPreparationErrors } from '../composables/useDeploymentPreparation'

defineProps<{
  templates: TemplateDto[]
  selectedTemplateName: string | null
  openApiSource: OpenApiSourceDto
  proxyName: string
  errors: ProxyCreationPreparationErrors
  ready: boolean
  preview: ProxyCreationPreviewDto | null
  logicalTargetEnvironment: string | null
  status: string
  generation: BundleGenerationResultDto | null
  createdRevision: CreatedProxyRevisionDto | null
  error: string | null
}>()

const fileError = ref<string | null>(null)

const emit = defineEmits<{
  selectTemplate: [name: string]
  updateOpenApiDisplayName: [value: string]
  updateOpenApiContent: [value: string]
  updateProxyName: [value: string]
  generate: []
  upload: []
  cancel: []
}>()

async function loadOpenApiFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  fileError.value = null
  try {
    emit('updateOpenApiDisplayName', file.name)
    emit('updateOpenApiContent', await file.text())
  } catch {
    fileError.value = 'The OpenAPI file could not be read.'
  }
}
</script>

<template>
  <div class="deployment-preparation">
    <label class="deployment-preparation__template-select">
      <span>Template source</span>
      <select :value="selectedTemplateName || ''" aria-label="Select proxy template" @change="$emit('selectTemplate', ($event.target as HTMLSelectElement).value)">
        <option value="">Select a saved template</option>
        <option v-for="template in templates" :key="template.name" :value="template.name">{{ template.name }}</option>
      </select>
    </label>
    <div class="deployment-preparation__header">
      <div>
        <h2>Prepare proxy creation</h2>
        <p>Review the local inputs before generating and uploading a proxy revision to Apigee.</p>
      </div>
      <BaseChip :label="ready ? 'Ready to generate' : 'Needs attention'" :tone="ready ? 'success' : 'warning'" />
    </div>
    <div class="deployment-preparation__form">
      <label>
        <span>OpenAPI display name</span>
        <input :value="openApiSource.display_name" type="text" placeholder="openapi.yaml" @input="$emit('updateOpenApiDisplayName', ($event.target as HTMLInputElement).value)" />
        <input class="deployment-preparation__file-input" type="file" accept=".yaml,.yml,.json,text/yaml,application/json" aria-label="Import OpenAPI file" @change="loadOpenApiFile" />
        <small v-if="fileError" class="deployment-preparation__field-error">{{ fileError }}</small>
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
    <div v-if="error" class="deployment-preparation__errors" role="alert" aria-live="assertive">
      <strong>Proxy creation failed</strong>
      <span>{{ error }}</span>
    </div>
    <div v-if="createdRevision" class="deployment-preparation__created" role="status" aria-live="polite">
      Proxy <strong>{{ createdRevision.proxy_name }}</strong> revision <strong>{{ createdRevision.revision }}</strong> was created and is <strong>Not deployed</strong>.
    </div>
    <div class="review-actions">
      <button type="button" @click="$emit('cancel')">Back to templates</button>
      <button type="button" class="primary-action" :disabled="!ready || status === 'generating' || status === 'uploading'" @click="$emit('generate')">
        {{ status === 'generating' ? 'Generating…' : generation ? 'Regenerate bundle' : 'Generate bundle' }}
      </button>
      <button v-if="generation && !createdRevision" type="button" :disabled="status === 'uploading'" @click="$emit('upload')">
        {{ status === 'uploading' ? 'Uploading…' : 'Upload and create proxy' }}
      </button>
    </div>
    <p v-if="generation" class="deployment-preparation__next-step">Bundle ready: {{ generation.rendered_file_count }} rendered files. Upload is a separate Apigee mutation and does not deploy the revision.</p>
    <p v-else class="deployment-preparation__next-step">Generation is local. This step does not call Apigee until you explicitly upload the bundle.</p>
  </div>
</template>
