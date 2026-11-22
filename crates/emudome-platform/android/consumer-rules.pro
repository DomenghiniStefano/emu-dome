# Called from Rust through JNI (attach) and implemented in Rust (native methods).
-keep class it.domehub.emudome.platform.NativeSurface { *; }
-keep class it.domehub.emudome.platform.SystemBridge { *; }
-keep class it.domehub.emudome.platform.GamepadInput { *; }
