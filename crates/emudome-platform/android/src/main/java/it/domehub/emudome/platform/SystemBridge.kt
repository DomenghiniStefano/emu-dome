package it.domehub.emudome.platform

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback

/**
 * What the system asks of EmuDome outside the page, and the games it hands over as documents.
 * Rust (crates/emudome-platform/src/android.rs) registers [nativeBackPressed] and calls [attach]
 * on the UI thread once the WebView exists; the document calls come from any thread.
 *
 * - Back: the back button / gesture goes to Rust, which routes it as the controller's menu
 *   button (in game) or its B (anywhere else).
 * - Games as `content://` documents: «Open with» at start ([startupGame]) and [openDocument],
 *   which gives Rust a file descriptor (no copy: a 3DS game is GBs).
 */
object SystemBridge {
    private const val TAG = "EmuDome"

    /** The back button or gesture, on the UI thread. */
    @JvmStatic
    external fun nativeBackPressed()

    @JvmStatic
    fun attach(activity: Activity) {
        val owner = activity as? ComponentActivity ?: run {
            Log.e(TAG, "not a ComponentActivity: the back button keeps the system's default")
            return
        }
        // The dispatcher asks the newest enabled callback first. Tauri's app plugin adds its own
        // (WebView history, else the activity's default) when it loads, which is expected before
        // the WebView reaches Rust and this call: to be confirmed on a device (Trello #14).
        owner.onBackPressedDispatcher.addCallback(owner, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() = nativeBackPressed()
        })
    }

    /** The game the app was started with: the `game` extra (adb), else the document of «Open with». */
    @JvmStatic
    fun startupGame(activity: Activity): String? {
        val intent = activity.intent ?: return null
        val extra = intent.getStringExtra("game")
        if (extra != null) return extra
        return if (intent.action == Intent.ACTION_VIEW) intent.data?.toString() else null
    }

    /**
     * Opens [uri] for reading: a file descriptor the caller owns and closes, or -1 (logged).
     * Takes the persistable permission first when one was granted, so the game opens again after
     * a restart.
     */
    @JvmStatic
    fun openDocument(context: Context, uri: String): Int {
        return try {
            val parsed = Uri.parse(uri)
            keepPermission(context, parsed)
            context.contentResolver.openFileDescriptor(parsed, "r")?.detachFd() ?: -1
        } catch (e: Exception) {
            Log.w(TAG, "document not opened: $uri: $e")
            -1
        }
    }

    /** The document's file name as its provider tells it, or null. */
    @JvmStatic
    fun displayName(context: Context, uri: String): String? {
        return try {
            context.contentResolver
                .query(Uri.parse(uri), arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
                ?.use { cursor -> if (cursor.moveToFirst()) cursor.getString(0) else null }
        } catch (e: Exception) {
            Log.w(TAG, "document name not read: $uri: $e")
            null
        }
    }

    /**
     * Keeps the read permission across restarts. Only a document picked with
     * `ACTION_OPEN_DOCUMENT` carries a persistable one; the others (the interface's picker today,
     * `ACTION_GET_CONTENT`, and most «Open with») are read only while the app runs.
     */
    private fun keepPermission(context: Context, uri: Uri) {
        try {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        } catch (e: SecurityException) {
            Log.i(TAG, "no persistable permission for $uri")
        }
    }
}
