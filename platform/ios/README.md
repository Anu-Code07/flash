# Flash iOS Backend

Mobile-first native UI via UIKit.

## Integration

```
Rust static lib (aarch64-apple-ios)
    ↓ C vtable
Swift/UIKit adapter
    ↓
UILabel, UIButton, UIStackView
```

## Frame driver

`CADisplayLink` — one `commit()` per vsync.

## Native renderer (Phase 3)

Implementation: `platform/ios/FlashHost/FlashHost.swift`

```swift
// AppDelegate.swift
FlashHost.shared.registerWithRust()
```

Rust calls `flash_host_apply_ops()` each frame with a batched command buffer.
One FFI crossing per vsync — no per-property bridge calls.

## Primitives (10 core widgets)

| Flash | UIKit |
|-------|-------|
| Text | UILabel |
| Button | UIButton |
| Column / Row | UIStackView |
| Stack | UIView (z-order) |
| Image | UIImageView |
| TextField | UITextField |
| ScrollView | UIScrollView |
| List | UICollectionView |
| Loading | UIActivityIndicatorView |

## Animation (Phase 5a)

Per-frame `set_prop` is **not** the animation API. Animations compile to
`CABasicAnimation` / `UIViewPropertyAnimator` — one setup call, zero crossings
during the animation. Only compositor properties: opacity, transform, offset.

## Layout (Phase 5b)

`UIStackView` + Auto Layout loses on deep trees. `FlashLayoutView` runs flex
layout in Rust and assigns `frame` directly, bypassing constraint solving.
