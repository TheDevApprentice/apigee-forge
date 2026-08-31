<script setup lang="ts">
import { computed, ref } from 'vue'
import BaseCard from '../base/BaseCard.vue'
import BaseChip from '../base/BaseChip.vue'
import supportContent from '../../data/support-content.json'
import type { SupportContent } from '../../types/support'

defineProps<{ authenticated?: boolean }>()
const emit = defineEmits<{ navigate: [view: string]; openArticle: [id: string] }>()
const content = supportContent as SupportContent
const articles = content.articles
const faqs = content.faqs
const form = ref({ email: '', category: 'General question', subject: '', message: '' })
const submitted = ref(false)
const formError = ref('')
const formValid = computed(() => form.value.email.trim() && form.value.subject.trim() && form.value.message.trim())

function submitSupport() {
  submitted.value = false
  formError.value = ''
  if (!formValid.value) {
    formError.value = 'Complete your email, subject and message before sending.'
    return
  }
  submitted.value = true
}
</script>

<template>
  <div class="support-page" aria-labelledby="support-title">
    <section class="support-hero support-reveal">
      <div><p class="support-eyebrow">Support centre</p><h1 id="support-title">A little help goes a long way.</h1><p>Find your way around Apigee Forge, understand each step and get back to building with confidence.</p></div>
      <div class="support-hero__illustration" aria-hidden="true"><span>?</span><i></i><i></i><i></i></div>
    </section>

    <section class="support-section support-reveal" aria-labelledby="support-topics-title">
      <div class="support-section__heading"><div><p class="support-eyebrow">Explore the essentials</p><h2 id="support-topics-title">How can we help?</h2></div><BaseChip :label="authenticated ? 'Workspace connected' : 'Available before sign-in'" :tone="authenticated ? 'success' : 'accent'" /></div>
      <div class="support-topic-grid"><button v-for="article in articles.slice(0, 3)" :key="article.id" type="button" class="support-topic" @click="emit('openArticle', article.id)"><span class="support-topic__icon">{{ article.id === 'getting-started' ? '?' : article.id === 'templates' ? '◇' : '↗' }}</span><span><strong>{{ article.title }}</strong><small>{{ article.summary }}</small></span><b>→</b></button></div>
    </section>

    <section class="support-section support-reveal" aria-labelledby="support-faq-title"><div class="support-section__heading"><div><p class="support-eyebrow">Answers, simply</p><h2 id="support-faq-title">Frequently asked questions</h2></div></div><div class="support-faq-grid"><details v-for="faq in faqs" :key="faq.id" class="support-faq"><summary>{{ faq.question }}<span aria-hidden="true">+</span></summary><p>{{ faq.answer }}</p></details></div></section>

    <section class="support-section support-contact-form support-reveal" aria-labelledby="support-contact-title"><div class="support-section__heading"><div><p class="support-eyebrow">Talk to the team</p><h2 id="support-contact-title">Need a hand with your workspace?</h2><p>Send a question and keep your context close. This form is ready for the support integration.</p></div><BaseChip label="Support request" tone="accent" /></div><form v-if="!submitted" class="support-form" @submit.prevent="submitSupport"><label><span>Email</span><input v-model="form.email" type="email" autocomplete="email" placeholder="you@example.com" /></label><label><span>Category</span><select v-model="form.category"><option>General question</option><option>Authentication</option><option>Templates</option><option>Proxy delivery</option><option>Deployment</option></select></label><label class="support-form__wide"><span>Subject</span><input v-model="form.subject" type="text" placeholder="What can we help with?" /></label><label class="support-form__wide"><span>Message</span><textarea v-model="form.message" rows="5" placeholder="Tell us what you are trying to do…"></textarea></label><p v-if="formError" class="support-form__error" role="alert">{{ formError }}</p><button type="submit" class="primary-action">Send support request</button></form><div v-else class="support-form__success" role="status"><span>✓</span><div><strong>Your request is ready to send.</strong><p>Thanks for sharing the context. The support connection can be wired to your preferred channel next.</p></div><button type="button" @click="submitted = false">Send another</button></div></section>

    <section class="support-contact support-reveal" aria-labelledby="support-contact-links-title"><div><p class="support-eyebrow">Still need a hand?</p><h2 id="support-contact-links-title">Keep the flow moving.</h2><p>Check the project documentation and Apigee references for deeper guidance.</p></div><div class="support-contact__links"><a href="https://github.com/TheDevApprentice/apigee-forge" target="_blank" rel="noreferrer">Project documentation <span>↗</span></a><a href="https://cloud.google.com/apigee/docs" target="_blank" rel="noreferrer">Apigee documentation <span>↗</span></a></div></section>
  </div>
</template>
