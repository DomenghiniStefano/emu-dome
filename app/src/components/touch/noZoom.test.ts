import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { cwd } from 'node:process'
import { describe, expect, it } from 'vitest'

/** A file of `app/`, where Vitest runs. */
const appFile = (path: string) => readFileSync(resolve(cwd(), path), 'utf8')

function viewportContent(html: string): string[] {
  const doc = new DOMParser().parseFromString(html, 'text/html')
  const content = doc.querySelector('meta[name="viewport"]')?.getAttribute('content') ?? ''
  return content.split(',').map(part => part.trim().replace(/\s+/g, ''))
}

/**
 * The page never zooms on a phone: the touch controls, the native game surface under the webview
 * and the touch on a DS screen are placed at zoom 1, so a pinch or a double tap would leave them
 * off from what is drawn. Three places hold it; each is read here as the file it is.
 */
describe('no zoom on a phone', () => {
  it('the viewport refuses pinch zoom and keeps the notch handling', () => {
    const parts = viewportContent(appFile('index.html'))
    expect(parts).toContain('user-scalable=no')
    expect(parts).toContain('maximum-scale=1')
    expect(parts).toContain('viewport-fit=cover')
  })

  it('the root refuses double-tap zoom', () => {
    const root = /html,\s*body,\s*#app\s*\{([^}]*)\}/.exec(appFile('src/styles.css'))?.[1] ?? ''
    expect(root).toMatch(/touch-action:\s*manipulation;/)
  })

  it('the Android WebView has zoom off', () => {
    const activity = appFile(
      'src-tauri/gen/android/app/src/main/java/it/domehub/emudome/MainActivity.kt',
    )
    expect(activity).toContain('override fun onWebViewCreate(webView: WebView)')
    expect(activity).toContain('setSupportZoom(false)')
    expect(activity).toContain('builtInZoomControls = false')
  })
})
