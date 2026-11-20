import { createApp } from 'vue'
import App from './App.vue'
import { i18n, startLanguage } from '@/i18n'
import { startAppearance } from '@/lib/appearance'
import { startTheme } from '@/lib/theme'
import { DevRoute } from '@/lib/devRoutes'
import { catchEverything } from '@/lib/errors'
import '@/assets/main.css'
import './styles.css'

startTheme()
startAppearance()
startLanguage()

// The Kit page (every primitive in every state) exists only in development: behind
// import.meta.env.DEV the production build drops the dynamic import, and src/kit/ with it.
if (import.meta.env.DEV && window.location.hash === DevRoute.Kit) {
  void import('@/kit/KitPage.vue').then(({ default: KitPage }) =>
    createApp(KitPage).use(i18n).mount('#app'),
  )
} else {
  const app = createApp(App).use(i18n)
  catchEverything(app)
  app.mount('#app')
}
