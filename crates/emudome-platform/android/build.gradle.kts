// EmuDome's Android glue: the SurfaceView under the WebView, the back button, games as documents
// (see ../src/android.rs), the controllers (../src/pads/android.rs).
// Included in the app's Gradle project by tauri-build (DEP_EMUDOME_PLATFORM_ANDROID_LIBRARY_PATH).
import groovy.json.JsonSlurper
import java.security.MessageDigest
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

// The libretro cores packaged in the APK: exactly the files cores.lock (repo root) lists for an
// Android ABI of a shipped core, sha256-checked, copied into build/emudome-cores/ (jniLibs/<abi>/
// for the libraries, assets/cores/ for their licences and the source offer). Nothing else in
// cores/android/ is packaged. A file with the wrong hash fails every build; a missing file fails a
// release build and only warns in a debug one (development without cores keeps working).
// scripts/fetch-cores-android.ps1 puts the files in place (it downloads the cores cores.lock pins
// as upstream builds and builds the ones it marks `built`, mGBA and melonDS DS, from source: the hashes are checked
// the same way); scripts/cores.mjs is the desktop side.
val repoRoot: File = projectDir.resolve("../../..").canonicalFile
val coresOut: File = layout.buildDirectory.dir("emudome-cores").get().asFile

data class PackagedFile(val from: File, val into: String, val sha256: String?)

@Suppress("UNCHECKED_CAST")
fun packagedCoreFiles(): List<PackagedFile> {
    val lock = JsonSlurper().parse(repoRoot.resolve("cores.lock")) as Map<String, Any?>
    val cores = lock["cores"] as Map<String, Map<String, Any?>>
    val fixed = listOf(
        PackagedFile(repoRoot.resolve("LICENSE"), "assets/LICENSE.txt", null),
        PackagedFile(repoRoot.resolve("cores/SOURCES.md"), "assets/cores/SOURCES.md", null),
    )
    val perCore = cores.values.filter { it["ship"] == true }.flatMap { core ->
        val license = core["license"] as Map<String, String>
        val builds = core["builds"] as Map<String, Map<String, Any?>>
        builds.filterKeys { it.startsWith("android-") }.flatMap { (platform, build) ->
            val abi = platform.removePrefix("android-")
            val notices = repoRoot.resolve("cores/notices/$platform")
            val library = build["library"] as Map<String, String>
            val extra = build["notices"] as List<Map<String, String>>
            listOf(
                PackagedFile(
                    repoRoot.resolve("cores/android/$abi/${library["file"]}"),
                    "jniLibs/$abi/${library["file"]}",
                    library["sha256"],
                ),
                PackagedFile(
                    notices.resolve(license.getValue("file")),
                    "assets/cores/notices/${license["file"]}",
                    license["sha256"],
                ),
            ) + extra.map {
                PackagedFile(notices.resolve(it.getValue("file")), "assets/cores/notices/${it["file"]}", it["sha256"])
            }
        }
    }
    return fixed + perCore
}

fun sha256Of(file: File): String =
    MessageDigest.getInstance("SHA-256").digest(file.readBytes()).joinToString("") { "%02x".format(it) }

val releaseRequested = gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }

val emudomeCores = tasks.register("emudomeCores") {
    description = "Copies the cores listed in cores.lock into the APK sources, sha256-checked."
    inputs.file(repoRoot.resolve("cores.lock"))
    inputs.dir(repoRoot.resolve("cores")).optional()
    outputs.dir(coresOut)
    doLast {
        coresOut.deleteRecursively()
        val files = packagedCoreFiles()
        val missing = files.filter { !it.from.isFile }
        val wrong = files.filter { it.from.isFile && it.sha256 != null && sha256Of(it.from) != it.sha256 }
        if (wrong.isNotEmpty()) {
            throw GradleException(
                "Cores not matching cores.lock (run scripts/fetch-cores-android.ps1): " +
                    wrong.joinToString { it.from.path },
            )
        }
        if (missing.isNotEmpty()) {
            val message = "Cores missing (scripts/fetch-cores-android.ps1): " + missing.joinToString { it.from.path }
            if (releaseRequested) throw GradleException(message)
            logger.warn("warning: $message; this debug APK goes without them")
        }
        files.filter { it.from.isFile }.forEach { it.from.copyTo(coresOut.resolve(it.into), overwrite = true) }
    }
}

android {
    namespace = "it.domehub.emudome.platform"
    compileSdk = 36

    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDir(coresOut.resolve("jniLibs"))
            assets.srcDir(coresOut.resolve("assets"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
}

tasks.named("preBuild") { dependsOn(emudomeCores) }

dependencies {
    // SystemBridge.kt: the back dispatcher (ComponentActivity, OnBackPressedCallback). compileOnly: the app
    // already ships AndroidX Activity (app/src-tauri/gen/android/app/build.gradle.kts, the same
    // version), so this library adds nothing to the APK and never moves the app's version.
    compileOnly("androidx.activity:activity:1.10.1")
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}
