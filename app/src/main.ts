import { createApp } from 'vue'
import App from './App.vue'
import { i18n, startLanguage } from './i18n'
import { startTheme } from './theme'
import './styles.css'

startTheme()
startLanguage()
createApp(App).use(i18n).mount('#app')
