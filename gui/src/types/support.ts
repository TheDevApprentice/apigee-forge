export type SupportArticleSection = {
  heading: string
  paragraphs: string[]
  steps?: string[]
  note?: string
}

export type SupportArticle = {
  id: string
  category: string
  eyebrow: string
  title: string
  summary: string
  reading_time: string
  sections: SupportArticleSection[]
}

export type SupportFaq = {
  id: string
  question: string
  answer: string
}

export type SupportContent = {
  articles: SupportArticle[]
  faqs: SupportFaq[]
}
