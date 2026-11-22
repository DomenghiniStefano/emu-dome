package it.domehub.emudome.platform

import android.app.Activity
import android.content.Context
import android.hardware.input.InputManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.Window
import kotlin.math.roundToInt

/**
 * Physical controllers (Bluetooth / USB) for Rust's controller thread (Trello #47).
 * Rust (crates/emudome-platform/src/pads/android.rs) registers the native methods and calls
 * [attach] on the UI thread, right after SystemBridge.
 *
 * The activity window's [Window.Callback] is wrapped, so every key and generic motion event is
 * seen before the views: the WebView would otherwise take the d-pad and the face buttons.
 * Only the events of a gamepad or joystick device go to Rust, as raw key codes and axis values
 * (Rust decides what each one is); a key Rust does not read, and every event of any other device
 * (touch, the phone's own back key, keyboards, IME), goes on to the system untouched.
 * Devices come and go through [InputManager.InputDeviceListener]. Everything runs on the UI
 * thread.
 */
object GamepadInput {
    private const val TAG = "EmuDome"

    /** A pad connected: [ranges] holds min and max of each of the attach's axes, NaN if absent. */
    @JvmStatic
    external fun nativePadAdded(
        id: Int,
        name: String?,
        descriptor: String?,
        vendor: Int,
        product: Int,
        ranges: FloatArray,
        battery: Int,
        atStart: Boolean,
    )

    @JvmStatic
    external fun nativePadRemoved(id: Int)

    /** True when Rust reads the key: the event is consumed. */
    @JvmStatic
    external fun nativePadKey(id: Int, keyCode: Int, down: Boolean): Boolean

    /** [values] in the attach's axes order. True when Rust knows the pad: the event is consumed. */
    @JvmStatic
    external fun nativePadMotion(id: Int, values: FloatArray): Boolean

    /** The `MotionEvent.AXIS_*` Rust reads, in the order of the values it gets. */
    private var axes = IntArray(0)
    private var values = FloatArray(0)
    /** The pads announced to Rust. */
    private val known = HashSet<Int>()
    private var attached = false

    @JvmStatic
    fun attach(activity: Activity, axes: IntArray) {
        if (attached) return
        val window = activity.window
        val inner = window?.callback
        if (window == null || inner == null) {
            Log.e(TAG, "controllers: the activity has no window callback to read them from")
            return
        }
        attached = true
        this.axes = axes
        values = FloatArray(axes.size)
        window.callback = PadCallback(inner)
        val input = activity.getSystemService(Context.INPUT_SERVICE) as? InputManager
        if (input == null) {
            Log.e(TAG, "controllers: no InputManager, connections will not be followed")
        } else {
            input.registerInputDeviceListener(listener, Handler(Looper.getMainLooper()))
        }
        for (id in InputDevice.getDeviceIds()) {
            val device = InputDevice.getDevice(id) ?: continue
            if (isPad(device)) announce(device, atStart = true)
        }
    }

    /** The activity's own callback, with the pads' events taken first. */
    private class PadCallback(private val inner: Window.Callback) : Window.Callback by inner {
        override fun dispatchKeyEvent(event: KeyEvent): Boolean =
            onKey(event) || inner.dispatchKeyEvent(event)

        override fun dispatchGenericMotionEvent(event: MotionEvent): Boolean =
            onMotion(event) || inner.dispatchGenericMotionEvent(event)
    }

    private fun onKey(event: KeyEvent): Boolean {
        val device = event.device ?: return false
        if (!isPad(device)) return false
        val down = when (event.action) {
            KeyEvent.ACTION_DOWN -> true
            KeyEvent.ACTION_UP -> false
            else -> return false
        }
        if (device.id !in known) announce(device, atStart = false)
        return nativePadKey(device.id, event.keyCode, down)
    }

    private fun onMotion(event: MotionEvent): Boolean {
        if (!event.isFromSource(InputDevice.SOURCE_JOYSTICK)) return false
        if (event.actionMasked != MotionEvent.ACTION_MOVE) return false
        val device = event.device ?: return false
        if (!isPad(device)) return false
        if (device.id !in known) announce(device, atStart = false)
        for (i in axes.indices) values[i] = event.getAxisValue(axes[i])
        return nativePadMotion(device.id, values)
    }

    /**
     * A gamepad or a joystick, not a virtual device (the navigation bar's back). A keyboard or a
     * TV remote with arrow keys reports SOURCE_DPAD only: it stays the system's.
     */
    private fun isPad(device: InputDevice): Boolean {
        if (device.isVirtual) return false
        val sources = device.sources
        return (sources and InputDevice.SOURCE_GAMEPAD) == InputDevice.SOURCE_GAMEPAD ||
            (sources and InputDevice.SOURCE_JOYSTICK) == InputDevice.SOURCE_JOYSTICK
    }

    private fun announce(device: InputDevice, atStart: Boolean) {
        val ranges = FloatArray(axes.size * 2) { Float.NaN }
        for ((i, axis) in axes.withIndex()) {
            val range = device.getMotionRange(axis, InputDevice.SOURCE_JOYSTICK) ?: continue
            ranges[2 * i] = range.min
            ranges[2 * i + 1] = range.max
        }
        known.add(device.id)
        nativePadAdded(
            device.id,
            device.name,
            device.descriptor,
            device.vendorId,
            device.productId,
            ranges,
            battery(device),
            atStart,
        )
    }

    /** The charge, 0–100, or -1 when the device does not tell (or before Android 12). */
    private fun battery(device: InputDevice): Int {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) return -1
        return try {
            val state = device.batteryState
            val capacity = state.capacity
            if (!state.isPresent || capacity.isNaN()) -1 else (capacity * 100).roundToInt().coerceIn(0, 100)
        } catch (e: Exception) {
            Log.w(TAG, "controller battery not read: $e")
            -1
        }
    }

    private val listener = object : InputManager.InputDeviceListener {
        override fun onInputDeviceAdded(deviceId: Int) {
            val device = InputDevice.getDevice(deviceId) ?: return
            if (isPad(device) && deviceId !in known) announce(device, atStart = false)
        }

        override fun onInputDeviceRemoved(deviceId: Int) {
            if (known.remove(deviceId)) nativePadRemoved(deviceId)
        }

        override fun onInputDeviceChanged(deviceId: Int) {
            val device = InputDevice.getDevice(deviceId)
            if (device != null && isPad(device)) {
                if (deviceId !in known) announce(device, atStart = false)
            } else if (known.remove(deviceId)) {
                nativePadRemoved(deviceId)
            }
        }
    }
}
