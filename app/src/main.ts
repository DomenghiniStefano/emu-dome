import { createApp } from 'vue'
import App from './App.vue'
import { i18n, startLanguage } from './i18n'
import { startAppearance } from './appearance'
import { startTheme } from './theme'
import './styles.css'

startTheme()
startAppearance()
startLanguage()
createApp(App).use(i18n).mount('#app')
