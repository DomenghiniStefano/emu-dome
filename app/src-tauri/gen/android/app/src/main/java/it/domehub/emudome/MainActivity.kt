package it.domehub.emudome

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  // No zoom in the WebView, whatever the page says: the touch controls and the native game surface
  // under it are placed at zoom 1 (index.html's viewport says the same; this holds if it changes).
  override fun onWebViewCreate(webView: WebView) {
    webView.settings.setSupportZoom(false)
    webView.settings.builtInZoomControls = false
    webView.settings.displayZoomControls = false
  }
}
