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

## Primitives (Phase 4)

| Flash | Android |
|-------|---------|
| Text | TextView |
| Button | MaterialButton |
| Column / Row | FlashFlexLayout (Phase 5b; avoids double-measure) |
| List | RecyclerView |

## Animation (Phase 5a)

Animations use `ObjectAnimator` / `ViewPropertyAnimator` on compositor-only
properties. Never animate layout properties per-frame through JNI.

## Layout (Phase 5b)

Nested `LinearLayout` double-measures. `FlashFlexLayout` — single measure pass,
frames assigned from Rust layout IR.
