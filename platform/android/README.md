# Flash Android Backend

Mobile-first native UI via Android View system.

## Integration

```
Rust .so (aarch64-linux-android)
    ↓ JNI + direct ByteBuffer
Kotlin adapter
    ↓
TextView, MaterialButton, custom ViewGroup
```

## Frame driver

`Choreographer` — one `commit()` per vsync.

## Native renderer (Phase 3)

Implementation: `platform/android/flash-host/src/main/kotlin/com/flash/FlashHost.kt`

```kotlin
FlashHost.init(applicationContext)
// Rust .so calls FlashHost.applyOps(bytes) each frame
```

## Primitives (10 core widgets)

| Flash | Android |
|-------|---------|
| Text | TextView |
| Button | Button (Material) |
| Column / Row | LinearLayout |
| Stack | FrameLayout |
| Image | ImageView |
| TextField | EditText |
| ScrollView | ScrollView |
| List | RecyclerView |
| Loading | ProgressBar |

## Animation (Phase 5a)

Animations use `ObjectAnimator` / `ViewPropertyAnimator` on compositor-only
properties. Never animate layout properties per-frame through JNI.

## Layout (Phase 5b)

Nested `LinearLayout` double-measures. `FlashFlexLayout` — single measure pass,
frames assigned from Rust layout IR.
