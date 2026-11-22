import java.util.Properties
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// EmuDome: release signing from a keystore kept outside the repository (CLAUDE.md, licence rule 6:
// never a keystore or a password in the repo). The build reads four environment variables:
//   EMUDOME_KEYSTORE           path of the keystore (.jks / .keystore), outside this repository
//   EMUDOME_KEYSTORE_PASSWORD  its password
//   EMUDOME_KEY_ALIAS          the key's alias in it
//   EMUDOME_KEY_PASSWORD       the key's password
// A release build without all four, or with the keystore inside the repository, stops here with
// the reason. Debug builds never read them (Gradle signs those with the machine's debug key).
val repoRoot: File = rootDir.resolve("../../../..").canonicalFile
val signingVariables = listOf(
    "EMUDOME_KEYSTORE",
    "EMUDOME_KEYSTORE_PASSWORD",
    "EMUDOME_KEY_ALIAS",
    "EMUDOME_KEY_PASSWORD",
)
val signing: Map<String, String?> =
    signingVariables.associateWith { providers.environmentVariable(it).orNull?.takeIf(String::isNotEmpty) }
val releaseRequested = gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }
val releaseKeystore: File? = signing["EMUDOME_KEYSTORE"]?.let { File(it).canonicalFile }

if (releaseRequested) {
    val missing = signingVariables.filter { signing[it] == null }
    if (missing.isNotEmpty()) {
        throw GradleException(
            "Release signing: set ${missing.joinToString()} (a keystore outside the repository; see " +
                "app/src-tauri/gen/android/app/build.gradle.kts). Debug builds need none of them.",
        )
    }
    val keystore = releaseKeystore ?: throw GradleException("Release signing: EMUDOME_KEYSTORE is not set")
    if (!keystore.isFile) throw GradleException("Release signing: EMUDOME_KEYSTORE $keystore is not a file")
    if (keystore.path.startsWith(repoRoot.path + File.separator)) {
        throw GradleException("Release signing: the keystore $keystore is inside the repository; move it out")
    }
}

android {
    signingConfigs {
        create("release") {
            if (releaseKeystore != null) {
                storeFile = releaseKeystore
                storePassword = signing["EMUDOME_KEYSTORE_PASSWORD"]
                keyAlias = signing["EMUDOME_KEY_ALIAS"]
                keyPassword = signing["EMUDOME_KEY_PASSWORD"]
            }
        }
    }
    compileSdk = 37
    namespace = "it.domehub.emudome"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "it.domehub.emudome"
        minSdk = 26
        targetSdk = 37
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            if (releaseKeystore != null) signingConfig = signingConfigs.getByName("release")
            optimization {
               enable = true
            }
            proguardFiles(
                *fileTree(".") {
                  include("**/*.pro")
                  exclude("build/**")
                }.files.toTypedArray()
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    buildFeatures {
        buildConfig = true
    }
    // EmuDome: 16 KB pages (Google Play, apps targeting Android 15+). The native libraries stay
    // uncompressed in the APK / AAB, and AGP (8.5.1 and later) aligns them to 16 KB in the zip, so a
    // 16 KB-page device maps them in place. It is AGP's default for minSdk 23+; written down so that a
    // later change to minSdk or a copied template cannot turn it off silently. Check a built APK with
    // `zipalign -c -P 16 -v 4 <apk>` (SDK build-tools 35+). The libraries' own ELF alignment is the
    // other half: `.cargo/config.toml` for libemudome_app.so, cores.lock for the cores.
    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = file("tauri.build.gradle.kts"))
