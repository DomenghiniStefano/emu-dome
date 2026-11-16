package com.domenghini.emudome.platform

import android.app.Activity
import android.graphics.Color
import android.os.Build
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.ViewGroup
import android.view.WindowInsets
import android.view.WindowInsetsController
import android.webkit.WebView

/**
 * The game's native surface: a SurfaceView placed under Tauri's WebView, which is made
 * transparent so the game shows through wherever the page paints nothing.
 *
 * Rust calls [attach] on the UI thread once the WebView exists, after registering the
 * two native methods (crates/emudome-platform/src/android.rs). The Surface goes to Rust,
 * where wgpu draws on it from the video thread.
 */
object NativeSurface {
    @JvmStatic
    external fun nativeSurfaceChanged(surface: Surface, width: Int, height: Int)

    /** Returns only once Rust no longer uses the surface. */
    @JvmStatic
    external fun nativeSurfaceDestroyed()

    @JvmStatic
    fun attach(activity: Activity, webView: WebView) {
        val parent = webView.parent as? ViewGroup ?: return
        val view = SurfaceView(activity)
        view.holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {}

            override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                nativeSurfaceChanged(holder.surface, width, height)
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
                nativeSurfaceDestroyed()
            }
        })
        // Index 0: under the WebView. A SurfaceView draws behind its window by default,
        // and the window lets it through where the views above are transparent.
        parent.addView(
            view,
            0,
            ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT),
        )
        webView.setBackgroundColor(Color.TRANSPARENT)
        hideSystemBars(activity)
    }

    /** In game the whole screen is the game's: status and navigation bars come back with a swipe. */
    private fun hideSystemBars(activity: Activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            activity.window.insetsController?.let {
                it.hide(WindowInsets.Type.systemBars())
                it.systemBarsBehavior = WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            }
        }
    }
}
