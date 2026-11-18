// What the details' pure functions need from vue-i18n: `t` and `te` of `useI18n()`, passed in so
// the functions stay testable without a running app.
export interface Translator {
  t: (key: string, params?: Record<string, unknown>) => string
  te: (key: string) => boolean
}
