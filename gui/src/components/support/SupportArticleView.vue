<script setup lang="ts">
import BaseChip from '../base/BaseChip.vue'
import BaseBackButton from '../base/BaseBackButton.vue'
import type { SupportArticle } from '../../types/support'

defineProps<{ article: SupportArticle }>()
const emit = defineEmits<{ back: [] }>()
</script>

<template>
  <article class="support-article" aria-labelledby="support-article-title">
    <BaseBackButton class="support-reveal" label="Back to Support" @click="emit('back')" />
    <header class="support-article__hero support-reveal"><div><p class="support-eyebrow">{{ article.eyebrow }}</p><h1 id="support-article-title">{{ article.title }}</h1><p>{{ article.summary }}</p></div><BaseChip :label="article.reading_time" tone="accent" /></header>
    <div class="support-article__body">
      <section v-for="section in article.sections" :key="section.heading" class="support-article__section support-reveal"><p class="support-article__section-number">{{ String(article.sections.indexOf(section) + 1).padStart(2, '0') }}</p><div><h2>{{ section.heading }}</h2><p v-for="paragraph in section.paragraphs" :key="paragraph">{{ paragraph }}</p><ol v-if="section.steps" class="support-article__steps"><li v-for="step in section.steps" :key="step">{{ step }}</li></ol><p v-if="section.note" class="support-article__note"><strong>Good to know</strong>{{ section.note }}</p></div></section>
    </div>
  </article>
</template>
