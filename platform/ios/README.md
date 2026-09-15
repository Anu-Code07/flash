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

## Primitives (Phase 3)

| Flash | UIKit |
|-------|-------|
| Text | UILabel |
| Button | UIButton |
| Column / Row | UIStackView (MVP) → FlashLayoutView (Phase 5b) |
| List | UICollectionView |

## Animation (Phase 5a)

Per-frame `set_prop` is **not** the animation API. Animations compile to
`CABasicAnimation` / `UIViewPropertyAnimator` — one setup call, zero crossings
during the animation. Only compositor properties: opacity, transform, offset.

## Layout (Phase 5b)

`UIStackView` + Auto Layout loses on deep trees. `FlashLayoutView` runs flex
layout in Rust and assigns `frame` directly, bypassing constraint solving.
