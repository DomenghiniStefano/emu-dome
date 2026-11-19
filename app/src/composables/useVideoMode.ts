// The video path (native under the webview, or the webview's canvas) and the page's background
// that goes with it. The quick menu switches it; the settings model mirrors it.
import { ref, watch } from 'vue'
import { emu, type VideoMode } from '../emu'
import { pageBackground } from '../lib/shell/session'
import { liveVideoMode } from '../settingsModel'
import type { Toasts } from './useToasts'

export function useVideoMode(toasts: Toasts) {
  const videoMode = ref<VideoMode>('native')
  watch(videoMode, mode => (liveVideoMode.value = mode))
  const nativeAvailable = ref(false)

  const paintBackground = () =>
    (document.documentElement.style.background = pageBackground(videoMode.value))

  /** The mode Rust started in, and whether native video is possible here. */
  async function load() {
    ;[videoMode.value, nativeAvailable.value] = await emu.videoMode()
    paintBackground()
  }

  async function setVideoMode(mode: VideoMode) {
    try {
      await emu.setVideoMode(mode)
    } catch (e) {
      toasts.failWith(e)
      return
    }
    videoMode.value = mode
    paintBackground()
  }

  return { videoMode, nativeAvailable, load, setVideoMode }
}
