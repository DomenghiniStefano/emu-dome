// EmuDome's Android glue: the SurfaceView under the WebView (see ../src/android.rs).
// Included in the app's Gradle project by tauri-build (DEP_EMUDOME_PLATFORM_ANDROID_LIBRARY_PATH).
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.domenghini.emudome.platform"
    compileSdk = 36

    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }

    // The libretro cores for Android (scripts/fetch-cores-android.ps1), packaged in the APK.
    sourceSets {
        getByName("main") {
            jniLibs.srcDir("../../../cores/android")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}
