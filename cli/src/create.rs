//! `flash create` — clean-architecture app scaffold (Flutter/RN-style).

use std::fs;
use std::path::Path;

pub fn run_create(name: &str, target: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Error: '{}' already exists", name);
        std::process::exit(1);
    }

    scaffold_clean_arch(dir, name);

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

    print_success(name, target);
}

fn scaffold_clean_arch(dir: &Path, name: &str) {
    fs::create_dir_all(dir.join("ui/screens")).expect("ui/screens");
    fs::create_dir_all(dir.join("scripts")).expect("scripts");
    fs::create_dir_all(dir.join("crates/presentation/src")).expect("presentation");
    fs::create_dir_all(dir.join("crates/application/src")).expect("application");
    fs::create_dir_all(dir.join("crates/infrastructure/src")).expect("infrastructure");

    write_file(dir, "flash.toml", &flash_toml(name));
    write_file(dir, "Cargo.toml", WORKSPACE_CARGO);
    write_file(dir, "ui/screens/home.ui", HOME_UI);
    write_file(dir, "scripts/build-rust.sh", BUILD_RUST_SH);
    write_file(dir, "README.md", &project_readme(name));

    write_file(dir, "crates/presentation/Cargo.toml", PRESENTATION_CARGO);
    write_file(dir, "crates/presentation/src/lib.rs", PRESENTATION_RS);
    write_file(dir, "crates/application/Cargo.toml", APPLICATION_CARGO);
    write_file(dir, "crates/application/src/lib.rs", APPLICATION_RS);
    write_file(dir, "crates/infrastructure/Cargo.toml", INFRASTRUCTURE_CARGO);
    write_file(dir, "crates/infrastructure/src/lib.rs", INFRASTRUCTURE_RS);
}

fn scaffold_ios(dir: &Path) {
    let ios = dir.join("platform/ios");
    fs::create_dir_all(ios.join("FlashApp")).expect("ios FlashApp");
    fs::create_dir_all(ios.join("FlashHost")).expect("ios FlashHost");

    write_file(dir, "platform/ios/FlashApp/AppDelegate.swift", IOS_APP_DELEGATE);
    write_file(dir, "platform/ios/FlashApp/SceneDelegate.swift", IOS_SCENE_DELEGATE);
    write_file(dir, "platform/ios/FlashApp/Info.plist", IOS_INFO_PLIST);
    write_file(dir, "platform/ios/FlashHost/FlashHost.swift", IOS_FLASH_HOST);
    write_file(dir, "platform/ios/FlashApp.xcodeproj/project.pbxproj", IOS_PBXPROJ);
    write_file(dir, "platform/ios/README.md", IOS_README);
}

fn scaffold_android(dir: &Path) {
    let android = "platform/android";
    fs::create_dir_all(dir.join(format!("{}/app/src/main/java/com/flash/app", android)))
        .expect("android app");
    fs::create_dir_all(dir.join(format!("{}/flash-host/src/main/kotlin/com/flash", android)))
        .expect("flash-host");

    write_file(dir, &format!("{}/settings.gradle.kts", android), ANDROID_SETTINGS);
    write_file(dir, &format!("{}/build.gradle.kts", android), ANDROID_ROOT_BUILD);
    write_file(dir, &format!("{}/app/build.gradle.kts", android), ANDROID_APP_BUILD);
    write_file(dir, &format!("{}/app/src/main/AndroidManifest.xml", android), ANDROID_MANIFEST);
    write_file(
        dir,
        &format!("{}/app/src/main/java/com/flash/app/MainActivity.kt", android),
        ANDROID_MAIN_ACTIVITY,
    );
    write_file(
        dir,
        &format!("{}/flash-host/src/main/kotlin/com/flash/FlashHost.kt", android),
        ANDROID_FLASH_HOST,
    );
    write_file(dir, &format!("{}/README.md", android), ANDROID_README);
}

fn print_success(name: &str, target: &str) {
    println!();
    println!("⚡ Created Flash app '{}'", name);
    println!();
    println!("  {}/", name);
    println!("  ├── ui/screens/          # .ui screens");
    println!("  ├── crates/");
    println!("  │   ├── presentation/    # view models → .ui");
    println!("  │   ├── application/     # use cases");
    println!("  │   └── infrastructure/  # HTTP, storage");
    println!("  ├── platform/");
    match target {
        "ios" => println!("  │   └── ios/             # Xcode project"),
        "android" => println!("  │   └── android/         # Gradle project"),
        _ => {
            println!("  │   ├── ios/             # Xcode project");
            println!("  │   └── android/         # Gradle project");
        }
    }
    println!("  ├── flash.toml");
    println!("  └── scripts/build-rust.sh");
    println!();
    println!("Next:");
    println!("  cd {}", name);
    println!("  flash dev              # hot reload (reads flash.toml)");
    println!("  flash dev --native     # native hot reload");
    println!("  flash doctor           # verify toolchain");
}

fn flash_toml(name: &str) -> String {
    format!(
        r#"# Flash app manifest — https://github.com/Anu-Code07/flash

[app]
name = "{name}"
entry = "ui/screens/home.ui"

[sdk]
# Set by `install.sh` — path to Flash SDK clone
path = "${{FLASH_SDK}}"

[targets.ios]
triple = "aarch64-apple-ios"
static_lib = "platform/ios/build/libflash_runtime.a"

[targets.android]
triple = "aarch64-linux-android"
shared_lib = "platform/android/app/src/main/jniLibs/arm64-v8a/libflash_runtime.so"

[crates]
presentation = "crates/presentation"
application = "crates/application"
infrastructure = "crates/infrastructure"
"#,
        name = name
    )
}

fn project_readme(name: &str) -> String {
    format!(
        r#"# {name}

Flash app — clean architecture layout.

## Quick start

```bash
flash dev              # hot reload (uses flash.toml entry)
flash dev --native     # native command-buffer hot reload
flash doctor           # check toolchain
./scripts/build-rust.sh   # build native libs for device
```

## Structure

```
{name}/
├── ui/screens/           # declarative .ui files
├── crates/
│   ├── presentation/     # view models → .ui
│   ├── application/      # use cases
│   └── infrastructure/   # HTTP, storage
├── platform/
│   ├── ios/              # Xcode project
│   └── android/          # Gradle project
└── flash.toml
```
"#,
        name = name
    )
}

fn write_file(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write template file");
}

const WORKSPACE_CARGO: &str = r#"[workspace]
resolver = "2"
members = [
    "crates/presentation",
    "crates/application",
    "crates/infrastructure",
]
"#;

const PRESENTATION_CARGO: &str = r#"[package]
name = "presentation"
version = "0.1.0"
edition = "2021"

[dependencies]
application = { path = "../application" }
"#;

const PRESENTATION_RS: &str = r#"//! Presentation layer — view models wired to `.ui` screens.
//!
//! Use `#[ui_export]` when the compiler supports full binding (coming soon).

use application::IncrementCount;

/// Counter screen state — mirrors `ui/screens/home.ui`.
pub struct CounterViewModel {
    pub count: i64,
}

impl CounterViewModel {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count = IncrementCount::run(self.count);
    }
}
"#;

const APPLICATION_CARGO: &str = r#"[package]
name = "application"
version = "0.1.0"
edition = "2021"

[dependencies]
infrastructure = { path = "../infrastructure" }
"#;

const APPLICATION_RS: &str = r#"//! Application layer — use cases (no UI, no platform APIs).

/// Increment counter use case.
pub struct IncrementCount;

impl IncrementCount {
    pub fn run(current: i64) -> i64 {
        current + 1
    }
}
"#;

const INFRASTRUCTURE_CARGO: &str = r#"[package]
name = "infrastructure"
version = "0.1.0"
edition = "2021"
"#;

const INFRASTRUCTURE_RS: &str = r#"//! Infrastructure layer — HTTP, storage, platform bridges.

/// Placeholder HTTP client — replace with reqwest or ureq.
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        Self
    }
}
"#;

const HOME_UI: &str = r#"screen Home {
  @state count: int = 0

  Column {
    Text(text: "Count: {count}")
    Button(title: "Increment") {
      count++
    }
  }
}
"#;

const BUILD_RUST_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FLASH_SDK="${FLASH_SDK:-$HOME/.flash/sdk}"

if [[ ! -d "$FLASH_SDK/cli" ]]; then
  echo "Flash SDK not found at $FLASH_SDK"
  echo "Run: curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash"
  exit 1
fi

echo "Building app Rust crates..."
cd "$ROOT"
cargo build --workspace --release

echo "Building Flash runtime for iOS..."
rustup target add aarch64-apple-ios 2>/dev/null || true
(cd "$FLASH_SDK" && cargo build -p flash-runtime --release --target aarch64-apple-ios)
mkdir -p "$ROOT/platform/ios/build"
cp "$FLASH_SDK/target/aarch64-apple-ios/release/libflash_runtime.a" \
   "$ROOT/platform/ios/build/"

echo "Building Flash runtime for Android..."
rustup target add aarch64-linux-android 2>/dev/null || true
(cd "$FLASH_SDK" && cargo build -p flash-runtime --release --target aarch64-linux-android)
mkdir -p "$ROOT/platform/android/app/src/main/jniLibs/arm64-v8a"
cp "$FLASH_SDK/target/aarch64-linux-android/release/libflash_runtime.so" \
   "$ROOT/platform/android/app/src/main/jniLibs/arm64-v8a/"

echo "✓ Done. Open platform/ios/FlashApp.xcodeproj or platform/android in Android Studio."
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

const IOS_README: &str = r#"# Flash iOS

1. From app root: `./scripts/build-rust.sh`
2. Open `FlashApp.xcodeproj` in Xcode
3. Link `build/libflash_runtime.a`, Library Search Paths → `$(PROJECT_DIR)/build`
4. Other Linker Flags: `-ObjC`
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
    }
}
"#;

const ANDROID_FLASH_HOST: &str = include_str!("../../platform/android/flash-host/src/main/kotlin/com/flash/FlashHost.kt");

const ANDROID_README: &str = r#"# Flash Android

1. From app root: `./scripts/build-rust.sh`
2. Open this folder in Android Studio
3. Run on emulator or device
"#;
