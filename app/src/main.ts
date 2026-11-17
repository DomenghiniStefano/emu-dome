import { createApp } from 'vue'
import App from './App.vue'
import { i18n } from './i18n'
import { startTheme } from './theme'
import './styles.css'

startTheme()
createApp(App).use(i18n).mount('#app')
