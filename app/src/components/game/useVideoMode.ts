// The video path (native under the webview, or the webview's canvas) and the page's background
// that goes with it. The quick menu switches it; the settings rows read the same refs
// (`videoMode.ts`: the screen effects exist only on the Accelerated method, X05).
import { emu, type VideoMode } from '@/emu'
import { pageBackground } from './session'
import { nativeAvailable, videoMode } from '@/lib/videoMode'
import type { Toasts } from '@/composables/useToasts'

export function useVideoMode(toasts: Toasts) {
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
