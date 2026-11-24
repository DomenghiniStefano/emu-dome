package it.domehub.emudome.platform

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Log
import android.view.WindowInsets
import android.view.WindowInsetsController
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContract
import androidx.activity.result.contract.ActivityResultContracts
import java.io.FileNotFoundException

/**
 * What the system asks of EmuDome outside the page, and the games it hands over as documents.
 * Rust (crates/emudome-platform/src/android.rs) registers [nativeBackPressed] and calls [attach]
 * on the UI thread once the WebView exists; the document calls come from any thread.
 *
 * - Back: the back button / gesture goes to Rust, which routes it as the controller's menu
 *   button (in game) or its B (anywhere else). At the shell's root the interface asks for
 *   [moveToBack] (Rust's `move_to_background`).
 * - Games as `content://` documents: «Open with» at start ([startupGame]) and [openDocument],
 *   which gives Rust a file descriptor (no copy: a 3DS game is GBs).
 * - The system's pickers (Trello #47): a folder of games ([pickTree], `ACTION_OPEN_DOCUMENT_TREE`)
 *   or one game file ([pickDocument], `ACTION_OPEN_DOCUMENT`), answered through
 *   [nativeDocumentPicked]; a picked folder's documents listed one folder per call ([listTree]).
 */
object SystemBridge {
    private const val TAG = "EmuDome"

    /** The back button or gesture, on the UI thread. */
    @JvmStatic
    external fun nativeBackPressed()

    /** A picker's answer: the document or tree picked and its display name, or nulls (cancelled). */
    @JvmStatic
    external fun nativeDocumentPicked(uri: String?, name: String?)

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

    /**
     * Back at the shell's root (Trello #29): what the system's back does at an app's first
     * screen, without finishing the activity: the task goes behind the others, the app keeps
     * running (a game's data is already flushed by `onPause`). Called from any thread.
     */
    @JvmStatic
    fun moveToBack(activity: Activity) {
        activity.runOnUiThread { activity.moveTaskToBack(true) }
    }

    /**
     * The system bars (Trello #131, Rust's `set_immersive`): hidden while a game is open, the whole
     * screen the game's (a swipe shows them for a moment), shown again in the shell, which then
     * lays itself out inside their insets (the page's `env(safe-area-inset-*)`). Applied again
     * each time the window gets the focus back (the notification shade, another app, a system
     * dialog can leave the bars shown). Before Android 11 nothing changes: the bars stay as the
     * system shows them. Called from any thread.
     */
    @JvmStatic
    fun setImmersive(activity: Activity, on: Boolean) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
        immersive = on
        activity.runOnUiThread {
            if (!focusWatched) {
                focusWatched = true
                activity.window.decorView.viewTreeObserver.addOnWindowFocusChangeListener { focused ->
                    if (focused) applyBars(activity)
                }
            }
            applyBars(activity)
        }
    }

    /** Whether the bars are wanted hidden ([setImmersive]); read on the UI thread. */
    @Volatile
    private var immersive = false

    /** The window's focus listener is added once (UI thread only). */
    private var focusWatched = false

    private fun applyBars(activity: Activity) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
        val controller = activity.window.insetsController ?: return
        if (immersive) {
            controller.systemBarsBehavior = WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsets.Type.systemBars())
        } else {
            controller.show(WindowInsets.Type.systemBars())
        }
    }

    /** The game the app was started with: the `game` extra (adb), else the document of «Open with». */
    @JvmStatic
    fun startupGame(activity: Activity): String? {
        val intent = activity.intent ?: return null
        val extra = intent.getStringExtra("game")
        if (extra != null) return extra
        return if (intent.action == Intent.ACTION_VIEW) intent.data?.toString() else null
    }

    /** [openDocument]'s answers when it fails (Rust's `content::open_failure` reads them). */
    private const val OPEN_FAILED = -1
    private const val OPEN_NOT_FOUND = -2
    private const val OPEN_PERMISSION_DENIED = -3

    /**
     * Opens [uri] for reading: a file descriptor the caller owns and closes, or a negative code
     * (logged) saying why not: [OPEN_NOT_FOUND] the document is gone, [OPEN_PERMISSION_DENIED]
     * the system refused it (the permission was taken back), [OPEN_FAILED] anything else.
     * Takes the persistable permission first when one was granted, so the game opens again after
     * a restart.
     */
    @JvmStatic
    fun openDocument(context: Context, uri: String): Int {
        return try {
            val parsed = Uri.parse(uri)
            keepPermission(context, parsed)
            context.contentResolver.openFileDescriptor(parsed, "r")?.detachFd() ?: OPEN_FAILED
        } catch (e: FileNotFoundException) {
            Log.w(TAG, "document not opened: $uri: $e")
            OPEN_NOT_FOUND
        } catch (e: SecurityException) {
            Log.w(TAG, "document not opened: $uri: $e")
            OPEN_PERMISSION_DENIED
        } catch (e: Exception) {
            Log.w(TAG, "document not opened: $uri: $e")
            OPEN_FAILED
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
     * A one-shot vibration for a touch control's press (Trello #29, `touch.thapt`): [millis] long
     * at [amplitude] (1–255). A device without a vibrator, or a failure, only logs. Any thread.
     */
    @JvmStatic
    fun vibrate(context: Context, millis: Long, amplitude: Int) {
        try {
            val vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                context.getSystemService(VibratorManager::class.java)?.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                context.getSystemService(Vibrator::class.java)
            }
            if (vibrator == null || !vibrator.hasVibrator()) return
            vibrator.vibrate(VibrationEffect.createOneShot(millis, amplitude.coerceIn(1, 255)))
        } catch (e: Exception) {
            Log.w(TAG, "no vibration: $e")
        }
    }

    /** The picker shown now, unregistered once it answered or when another one replaces it. */
    private var picker: ActivityResultLauncher<*>? = null

    private const val PICKER_KEY = "emudome-document-picker"

    /**
     * Shows the system's folder picker (`ACTION_OPEN_DOCUMENT_TREE`). The folder picked keeps its
     * read permission across restarts (persisted); its tree URI and display name reach Rust
     * through [nativeDocumentPicked], from a background thread (the name is a provider query).
     * [start], a tree picked before (null: where the system chooses), is where the picker opens
     * (`EXTRA_INITIAL_URI`, set by the contract), so the same folder is picked again with one
     * tap (Trello #141). Any thread.
     */
    @JvmStatic
    fun pickTree(activity: Activity, start: String?) {
        val initial = start?.let { treeRoot(it) }
        showPicker(activity, ActivityResultContracts.OpenDocumentTree(), initial) { uri ->
            persistRead(activity, uri)
            uri.toString() to treeName(activity, uri.toString())
        }
    }

    /**
     * Shows the system's file picker (`ACTION_OPEN_DOCUMENT`) for one game file. ROM files have no
     * registered MIME types, so every file is offered (MIME type any); Rust checks the extension.
     * The document keeps its read permission across restarts (persisted), so a game opened from
     * it opens again from the library. Any thread.
     */
    @JvmStatic
    fun pickDocument(activity: Activity) {
        showPicker(activity, ActivityResultContracts.OpenDocument(), arrayOf("*/*")) { uri ->
            persistRead(activity, uri)
            uri.toString() to displayName(activity, uri.toString())
        }
    }

    private fun <I> showPicker(
        activity: Activity,
        contract: ActivityResultContract<I, Uri?>,
        input: I,
        answer: (Uri) -> Pair<String, String?>,
    ) {
        val owner = activity as? ComponentActivity ?: run {
            Log.e(TAG, "not a ComponentActivity: no picker")
            nativeDocumentPicked(null, null)
            return
        }
        activity.runOnUiThread {
            try {
                picker?.unregister()
                // Registered without a lifecycle owner (allowed once the activity started) and
                // unregistered by hand. The activity is not recreated on rotation
                // (configChanges in the app's manifest), so the answer comes back here.
                val launcher = owner.activityResultRegistry.register(PICKER_KEY, contract) { uri ->
                    picker?.unregister()
                    picker = null
                    Thread {
                        val picked = try {
                            uri?.let(answer)
                        } catch (e: Exception) {
                            Log.w(TAG, "picked document not read: $uri: $e")
                            null
                        }
                        nativeDocumentPicked(picked?.first, picked?.second)
                    }.start()
                }
                picker = launcher
                launcher.launch(input)
            } catch (e: Exception) {
                Log.e(TAG, "picker not shown: $e")
                nativeDocumentPicked(null, null)
            }
        }
    }

    /** Persists the read permission of a picked document or tree (logged when refused). */
    private fun persistRead(context: Context, uri: Uri) {
        try {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        } catch (e: SecurityException) {
            Log.w(TAG, "read permission not persisted for $uri (lost at the next start): $e")
        }
    }

    /**
     * The root folder of the tree [tree] as a document URI, the form the picker's
     * `EXTRA_INITIAL_URI` takes; null (the picker opens where the system chooses) when [tree] is
     * not a tree URI.
     */
    private fun treeRoot(tree: String): Uri? {
        return try {
            val treeUri = Uri.parse(tree)
            DocumentsContract.buildDocumentUri(
                treeUri.authority,
                DocumentsContract.getTreeDocumentId(treeUri),
            )
        } catch (e: Exception) {
            Log.w(TAG, "picker not opened on $tree: $e")
            null
        }
    }

    /** The display name of a picked folder (its root document), or null when it cannot be read. */
    @JvmStatic
    fun treeName(context: Context, tree: String): String? {
        return try {
            val treeUri = Uri.parse(tree)
            val root = DocumentsContract.buildDocumentUriUsingTree(
                treeUri,
                DocumentsContract.getTreeDocumentId(treeUri),
            )
            context.contentResolver
                .query(root, arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME), null, null, null)
                ?.use { cursor -> if (cursor.moveToFirst()) cursor.getString(0) else null }
        } catch (e: Exception) {
            Log.w(TAG, "folder name not read: $tree: $e")
            null
        }
    }

    /**
     * The children of one folder of a picked tree: [dir] is a document URI of the tree (one this
     * call returned), or null for the tree's root. One provider query per folder, flattened into
     * five strings per child: document URI, display name, size, last modified (Unix ms, "0" when
     * unknown), "d" for a folder or "f". Null when the folder cannot be listed (permission
     * revoked, folder deleted): Rust reports it unreachable. Not on the UI thread.
     */
    @JvmStatic
    fun listTree(context: Context, tree: String, dir: String?): Array<String>? {
        return try {
            val treeUri = Uri.parse(tree)
            val parent = if (dir == null) {
                DocumentsContract.getTreeDocumentId(treeUri)
            } else {
                DocumentsContract.getDocumentId(Uri.parse(dir))
            }
            val children = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, parent)
            val columns = arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_SIZE,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
            )
            context.contentResolver.query(children, columns, null, null, null)?.use { c ->
                val out = ArrayList<String>(c.count * 5)
                while (c.moveToNext()) {
                    val id = c.getString(0) ?: continue
                    out.add(DocumentsContract.buildDocumentUriUsingTree(treeUri, id).toString())
                    out.add(c.getString(1) ?: "")
                    out.add(if (c.isNull(2)) "0" else c.getLong(2).toString())
                    out.add(if (c.isNull(3)) "0" else c.getLong(3).toString())
                    out.add(if (c.getString(4) == DocumentsContract.Document.MIME_TYPE_DIR) "d" else "f")
                }
                out.toTypedArray()
            }
        } catch (e: Exception) {
            Log.w(TAG, "folder not listed: $tree ${dir ?: "(root)"}: $e")
            null
        }
    }

    /**
     * Whether the app still holds the persisted read permission of the picked tree [tree].
     * Android takes it back when the app is uninstalled, even with its data kept: the library
     * still has the tree URI, but listing it fails. Not on the UI thread.
     */
    @JvmStatic
    fun treePermitted(context: Context, tree: String): Boolean {
        val uri = Uri.parse(tree)
        return context.contentResolver.persistedUriPermissions.any { it.uri == uri && it.isReadPermission }
    }

    /** Gives back the persisted read permission of a folder that left the library. */
    @JvmStatic
    fun releaseTree(context: Context, tree: String) {
        try {
            context.contentResolver.releasePersistableUriPermission(
                Uri.parse(tree),
                Intent.FLAG_GRANT_READ_URI_PERMISSION,
            )
        } catch (e: Exception) {
            Log.i(TAG, "no permission to give back for $tree: $e")
        }
    }

    /**
     * Keeps the read permission across restarts. Only a document picked with
     * `ACTION_OPEN_DOCUMENT` carries a persistable one; the others (the interface's picker today,
     * `ACTION_GET_CONTENT`, and most «Open with») are read only while the app runs.
     */
    private fun keepPermission(context: Context, uri: Uri) {
        // A document of a picked folder reads through the folder's persisted permission.
        if (DocumentsContract.isTreeUri(uri)) return
        try {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        } catch (e: SecurityException) {
            Log.i(TAG, "no persistable permission for $uri")
        }
    }
}
