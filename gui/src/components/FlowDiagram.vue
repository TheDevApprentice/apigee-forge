<script setup lang="ts">
type FlowStage = {
  request?: unknown[]
  response?: unknown[]
}

type ConditionalFlow = FlowStage & {
  condition?: string
}

type FlowDraft = {
  pre_flow: FlowStage
  conditional_flows: ConditionalFlow[]
  post_flow: FlowStage
}

defineProps<{
  flow: FlowDraft
  selectedFlow: string
}>()

defineEmits<{
  selectStage: [stage: string]
  updateCondition: [index: number, condition: string]
  removeCondition: [index: number]
  addCondition: []
}>()

function policyCount(stage: FlowStage): number {
  return (Array.isArray(stage.request) ? stage.request.length : 0)
    + (Array.isArray(stage.response) ? stage.response.length : 0)
}

function stageLabel(selectedFlow: string): string {
  if (selectedFlow === 'pre_flow') return 'PreFlow'
  if (selectedFlow === 'post_flow') return 'PostFlow'
  return `Conditional Flow ${Number(selectedFlow.split('_')[1]) + 1}`
}
</script>

<template>
  <div class="flow-diagram" aria-label="Template flow stages">
    <button type="button" class="flow-stage" :class="{ 'flow-stage--selected': selectedFlow === 'pre_flow' }" @click="$emit('selectStage', 'pre_flow')">
      <strong>PreFlow</strong>
      <span>{{ policyCount(flow.pre_flow) }} policies</span>
    </button>
    <div v-for="(conditional, index) in flow.conditional_flows" :key="`conditional-${index}`" class="flow-stage flow-stage--conditional" :class="{ 'flow-stage--selected': selectedFlow === `conditional_${index}` }">
      <button type="button" class="flow-stage__main" @click="$emit('selectStage', `conditional_${index}`)">
        <strong>Conditional Flow {{ index + 1 }}</strong>
        <span>{{ policyCount(conditional) }} policies</span>
      </button>
      <input :value="conditional.condition || ''" placeholder="Condition" :aria-label="`Conditional Flow ${index + 1} condition`" @input="$emit('updateCondition', index, ($event.target as HTMLInputElement).value)" />
      <button type="button" class="flow-stage__remove" @click="$emit('removeCondition', index)">Remove</button>
    </div>
    <button type="button" class="flow-stage" :class="{ 'flow-stage--selected': selectedFlow === 'post_flow' }" @click="$emit('selectStage', 'post_flow')">
      <strong>PostFlow</strong>
      <span>{{ policyCount(flow.post_flow) }} policies</span>
    </button>
  </div>
  <div class="flow-canvas__actions">
    <button type="button" @click="$emit('addCondition')">Add conditional flow</button>
  </div>
  <div class="flow-stage-detail" role="status" aria-live="polite">
    <span>Selected stage</span>
    <strong>{{ stageLabel(selectedFlow) }}</strong>
    <span>Request: {{ (selectedFlow === 'pre_flow' ? flow.pre_flow : selectedFlow === 'post_flow' ? flow.post_flow : flow.conditional_flows[Number(selectedFlow.split('_')[1])])?.request?.length || 0 }} policies</span>
    <span>Response: {{ (selectedFlow === 'pre_flow' ? flow.pre_flow : selectedFlow === 'post_flow' ? flow.post_flow : flow.conditional_flows[Number(selectedFlow.split('_')[1])])?.response?.length || 0 }} policies</span>
  </div>
</template>
