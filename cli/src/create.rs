//! `flash create` — scaffold iOS/Android app linking Rust static lib.

use std::fs;
use std::path::Path;

pub fn run_create(name: &str, target: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Error: '{}' already exists", name);
        std::process::exit(1);
    }

    fs::create_dir_all(dir.join("ui")).expect("create ui dir");
    fs::create_dir_all(dir.join("scripts")).expect("create scripts dir");

    write_file(dir, "flash.toml", FLASH_TOML);
    write_file(dir, "ui/home.ui", HOME_UI);
    write_file(dir, "scripts/build-rust.sh", BUILD_RUST_SH);

    match target {
        "ios" => scaffold_ios(dir),
        "android" => scaffold_android(dir),
        "all" => {
            scaffold_ios(dir);
            scaffold_android(dir);
        }
        other => {
            eprintln!("unknown target: {} (use ios, android, or all)", other);
            std::process::exit(1);
        }
    }

    println!("Created Flash app '{}'", name);
    println!("  ui/home.ui       — edit your UI");
    println!("  flash.toml       — project config");
    println!("  scripts/build-rust.sh — build static lib for device/simulator");
    if target == "ios" || target == "all" {
        println!("  ios/             — open FlashApp.xcodeproj in Xcode");
    }
    if target == "android" || target == "all" {
        println!("  android/         — open in Android Studio");
    }
    println!("\nNext:");
    println!("  cd {} && flash dev ui/home.ui", name);
    println!("  cd {} && flash dev --native ui/home.ui", name);
}

fn scaffold_ios(dir: &Path) {
    fs::create_dir_all(dir.join("ios/FlashApp")).expect("create ios dir");
    fs::create_dir_all(dir.join("ios/FlashHost")).expect("create FlashHost dir");

    write_file(dir, "ios/FlashApp/AppDelegate.swift", IOS_APP_DELEGATE);
    write_file(dir, "ios/FlashApp/SceneDelegate.swift", IOS_SCENE_DELEGATE);
    write_file(dir, "ios/FlashApp/Info.plist", IOS_INFO_PLIST);
    write_file(dir, "ios/FlashHost/FlashHost.swift", IOS_FLASH_HOST);
    write_file(dir, "ios/FlashApp.xcodeproj/project.pbxproj", IOS_PBXPROJ);
    write_file(dir, "ios/README.md", IOS_README);
}

fn scaffold_android(dir: &Path) {
    fs::create_dir_all(dir.join("android/app/src/main/java/com/flash/app")).expect("android app");
    fs::create_dir_all(dir.join("android/flash-host/src/main/kotlin/com/flash")).expect("flash-host");

    write_file(dir, "android/settings.gradle.kts", ANDROID_SETTINGS);
    write_file(dir, "android/build.gradle.kts", ANDROID_ROOT_BUILD);
    write_file(dir, "android/app/build.gradle.kts", ANDROID_APP_BUILD);
    write_file(dir, "android/app/src/main/AndroidManifest.xml", ANDROID_MANIFEST);
    write_file(
        dir,
        "android/app/src/main/java/com/flash/app/MainActivity.kt",
        ANDROID_MAIN_ACTIVITY,
    );
    write_file(
        dir,
        "android/flash-host/src/main/kotlin/com/flash/FlashHost.kt",
        ANDROID_FLASH_HOST,
    );
    write_file(dir, "android/README.md", ANDROID_README);
}

fn write_file(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write template file");
}

const FLASH_TOML: &str = r#"[app]
name = "MyFlashApp"
entry = "ui/home.ui"

[targets.ios]
triple = "aarch64-apple-ios"
static_lib = "ios/build/libflash_runtime.a"

[targets.android]
triple = "aarch64-linux-android"
shared_lib = "android/app/src/main/jniLibs/arm64-v8a/libflash_runtime.so"
"#;

const HOME_UI: &str = r#"screen Counter {
  @state count: int = 0

  Column {
    Text(text: "{count}")
    Button(title: "Increment") {
      count++
    }
  }
}
"#;

const BUILD_RUST_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "Building flash-runtime for iOS..."
rustup target add aarch64-apple-ios-sim aarch64-apple-ios 2>/dev/null || true
cargo build -p flash-runtime --release --target aarch64-apple-ios
mkdir -p ios/build
cp ../target/aarch64-apple-ios/release/libflash_runtime.a ios/build/

echo "Building flash-runtime for Android (arm64)..."
rustup target add aarch64-linux-android 2>/dev/null || true
cargo build -p flash-runtime --release --target aarch64-linux-android
mkdir -p android/app/src/main/jniLibs/arm64-v8a
cp ../target/aarch64-linux-android/release/libflash_runtime.so \
   android/app/src/main/jniLibs/arm64-v8a/

echo "Done. Link ios/build/libflash_runtime.a in Xcode or run Android app."
"#;

const IOS_APP_DELEGATE: &str = r#"import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        FlashHost.shared.registerWithRust()
        return true
    }
}
"#;

const IOS_SCENE_DELEGATE: &str = r#"import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?

    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {
        guard let windowScene = scene as? UIWindowScene else { return }
        let window = UIWindow(windowScene: windowScene)
        let root = UIViewController()
        root.view = UIView(frame: windowScene.screen.bounds)
        root.view.backgroundColor = .systemBackground
        FlashHost.shared.rootView = root.view
        window.rootViewController = root
        window.makeKeyAndVisible()
        self.window = window
    }
}
"#;

const IOS_INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>$(DEVELOPMENT_LANGUAGE)</string>
    <key>CFBundleExecutable</key>
    <string>$(EXECUTABLE_NAME)</string>
    <key>CFBundleIdentifier</key>
    <string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$(PRODUCT_NAME)</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>UIApplicationSceneManifest</key>
    <dict>
        <key>UIApplicationSupportsMultipleScenes</key>
        <false/>
        <key>UISceneConfigurations</key>
        <dict>
            <key>UIWindowSceneSessionRoleApplication</key>
            <array>
                <dict>
                    <key>UISceneConfigurationName</key>
                    <string>Default Configuration</string>
                    <key>UISceneDelegateClassName</key>
                    <string>$(PRODUCT_MODULE_NAME).SceneDelegate</string>
                </dict>
            </array>
        </dict>
    </dict>
</dict>
</plist>
"#;

// Copied from platform/ios/FlashHost/FlashHost.swift — keep in sync.
const IOS_FLASH_HOST: &str = include_str!("../../platform/ios/FlashHost/FlashHost.swift");

const IOS_PBXPROJ: &str = r#"// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {};
	objectVersion = 56;
	objects = {
		APP001 /* FlashApp.app */ = {isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = FlashApp.app; sourceTree = BUILT_PRODUCTS_DIR; };
		SRC001 /* AppDelegate.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = AppDelegate.swift; sourceTree = "<group>"; };
		SRC002 /* SceneDelegate.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = SceneDelegate.swift; sourceTree = "<group>"; };
		SRC003 /* FlashHost.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = FlashHost.swift; sourceTree = "<group>"; };
		LIB001 /* libflash_runtime.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = libflash_runtime.a; path = build/libflash_runtime.a; sourceTree = "<group>"; };
		GRP001 = {isa = PBXGroup; children = (SRC001, SRC002); name = FlashApp; path = FlashApp; sourceTree = "<group>"; };
		GRP002 = {isa = PBXGroup; children = (SRC003); name = FlashHost; path = FlashHost; sourceTree = "<group>"; };
		GRP003 = {isa = PBXGroup; children = (GRP001, GRP002, LIB001); sourceTree = "<group>"; };
		TGT001 = {
			isa = PBXNativeTarget;
			buildConfigurationList = CFG001;
			buildPhases = (BLD001, LNK001);
			buildRules = ();
			dependencies = ();
			name = FlashApp;
			productName = FlashApp;
			productReference = APP001;
			productType = "com.apple.product-type.application";
		};
		BLD001 = {isa = PBXSourcesBuildPhase; buildActionMask = 2147483647; files = (); runOnlyForDeploymentPostprocessing = 0; };
		LNK001 = {
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = ();
			runOnlyForDeploymentPostprocessing = 0;
		};
		PRJ001 = {
			isa = PBXProject;
			buildConfigurationList = CFG001;
			compatibilityVersion = "Xcode 14.0";
			developmentRegion = en;
			hasScannedForEncodings = 0;
			mainGroup = GRP003;
			productRefGroup = GRP003;
			projectDirPath = "";
			projectRoot = "";
			targets = (TGT001);
		};
		CFG001 = {isa = XCConfigurationList; buildConfigurations = (); defaultConfigurationIsVisible = 0; defaultConfigurationName = Release; };
	};
	rootObject = PRJ001;
}
"#;

const IOS_README: &str = r#"# Flash iOS app

1. Run `../scripts/build-rust.sh` from the repo root (links `libflash_runtime.a`).
2. Open `FlashApp.xcodeproj` in Xcode.
3. Add `FlashHost.swift`, `AppDelegate.swift`, `SceneDelegate.swift` to the target if not auto-linked.
4. Link `build/libflash_runtime.a` + set Library Search Paths to `$(PROJECT_DIR)/build`.
5. Other Linker Flags: `-ObjC`
6. `FlashHost.shared.registerWithRust()` runs in AppDelegate — UI mounts via Rust → UIKit.
"#;

const ANDROID_SETTINGS: &str = r#"pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "FlashApp"
include(":app")
include(":flash-host")
"#;

const ANDROID_ROOT_BUILD: &str = r#"plugins {
    id("com.android.application") version "8.2.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.20" apply false
}
"#;

const ANDROID_APP_BUILD: &str = r#"plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}
android {
    namespace = "com.flash.app"
    compileSdk = 34
    defaultConfig {
        applicationId = "com.flash.app"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
    }
    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }
}
dependencies {
    implementation(project(":flash-host"))
}
"#;

const ANDROID_MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:allowBackup="true"
        android:label="FlashApp"
        android:supportsRtl="true"
        android:theme="@android:style/Theme.Material.Light">
        <activity
            android:name=".MainActivity"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#;

const ANDROID_MAIN_ACTIVITY: &str = r#"package com.flash.app

import android.os.Bundle
import android.widget.FrameLayout
import androidx.appcompat.app.AppCompatActivity
import com.flash.FlashHost

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        FlashHost.init(this)
        val root = FrameLayout(this)
        FlashHost.rootView = root
        setContentView(root)
        // Rust runtime mounts UI via JNI → FlashHost.applyOps()
    }
}
"#;

const ANDROID_FLASH_HOST: &str = include_str!("../../platform/android/flash-host/src/main/kotlin/com/flash/FlashHost.kt");

const ANDROID_README: &str = r#"# Flash Android app

1. Run `../scripts/build-rust.sh` — copies `libflash_runtime.so` to `jniLibs/arm64-v8a/`.
2. Open `android/` in Android Studio.
3. `FlashHost.init(context)` + `FlashHost.rootView` set in MainActivity.
4. Rust `.so` calls `FlashHost.applyOps()` each frame; taps call `nativeFireHandler`.
"#;
