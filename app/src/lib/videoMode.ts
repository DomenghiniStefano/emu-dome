// Where the game is drawn, shared by the screens that depend on it: `useVideoMode` sets it
// (startup, the quick menu's «Metodo di visualizzazione»), the settings rows read it (the
// screen effects exist only on the Accelerated method, X05).
import { ref } from 'vue'
import type { VideoMode } from '@/emu'

/** The method in use: `native` = Accelerato, `webview` = Compatibile. */
export const videoMode = ref<VideoMode>('native')
/** Whether this system has the Accelerated method at all. */
export const nativeAvailable = ref(false)
