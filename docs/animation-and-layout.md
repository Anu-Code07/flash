# Flash: Animation and Layout — Known Weaknesses and Mitigations

Mobile-first targets: iOS (UIKit), Android (View), Web (DOM).

This document records where Flash loses to Flutter today, and the architectural
fixes required before benchmarks are run.

---

## 1. Animation — the worst metric

### The problem

Animating 1000 nodes naively means 1000 property writes per frame. The command
buffer collapses FFI to **one boundary crossing per frame**, which is correct —
but on the far side UIKit still invalidates Auto Layout and Android still calls
`requestLayout()` at 60 Hz. Flutter owns its renderer and raster thread and does
none of that. **Flutter will beat Flash on animation benchmarks, possibly badly.**

### The rule

> **Never drive animations through per-frame reactive flush.**

Per-frame `set_prop` → `commit` is for discrete state updates (counter text,
list item label). It is not an animation API.

### The fix: platform-native animators

Compile `.withAnimation` / `.animate` into **static animation descriptions** that
run entirely on the platform render thread with **zero per-frame crossings**.

```
.ui source
    ↓
Compiler identifies animatable properties (opacity, transform, offset)
    ↓
UI IR: AnimationSpec (not UpdateOp)
    ↓
Codegen: one-shot native animation setup
    ↓
iOS:     CABasicAnimation / UIViewPropertyAnimator
Android: ObjectAnimator / ViewPropertyAnimator
Web:     Web Animations API (WAAPI)
```

#### Animatable vs non-animatable properties

| Animatable (compositor-only) | Never animate (triggers layout) |
|------------------------------|----------------------------------|
| `opacity`                    | `width`, `height`                |
| `transform` (translate, scale, rotate) | `padding`, `margin`   |
| `offset` (visual only, not layout) | `fontSize`, `text`       |

The compiler emits **UI4001** if a developer tries to animate a layout property:

```
error[UI4001]: cannot animate `fontSize` — layout property
  = help: animate `opacity` or `transform` instead
  = note: layout animations require Phase 5b custom layout engine
```

#### Syntax (proposed)

```text
Column {
    Box()
        .opacity(show ? 1.0 : 0.0)
        .withAnimation(.easeInOut(duration: 300))

    // Or scoped:
    withAnimation(.spring) {
        if expanded {
            DetailPanel()
        }
    }
}
```

#### IR representation

```text
ANIMATION_SPEC anim0
  nodes: [Box#3]
  property: opacity
  from: slot(expanded) → 0.0 | 1.0     // compiled lookup, not runtime interp
  duration: 300ms
  curve: easeInOut
  driver: platform                       // NOT reactive

DEPENDENCIES
  slot(expanded) → [anim0]              // triggers animation restart, not set_prop
```

When `expanded` changes, the runtime calls `renderer.start_animation(anim0)` once.
No property writes until the animation completes (optional completion callback).

#### Benchmark target

1000 nodes animating opacity: **0 boundary crossings during animation** (one setup
call per node or batched setup). Target: within 2× of Flutter on same device, not 10×.

---

## 2. Complex layouts — second weakness

### The problem

| Platform | Default layout | Cost |
|----------|----------------|------|
| iOS | `UIStackView` + Auto Layout | Constraint solving per invalidation |
| Android | nested `LinearLayout` | Double measure pass |
| Flutter | single-pass flex in renderer | One layout pass, no platform solver |

Deep trees: **Flutter is genuinely faster.** Flash loses until custom layout
views exist.

### The fix: move custom layout from Phase 8 → Phase 5b

Custom layout must land **before** the `flights` example (deep lists, nested
rows), not after GPU investigation.

#### Phase 5b deliverables

| Target | Implementation |
|--------|----------------|
| iOS | `FlashLayoutView` — flex layout in Rust, assigns `frame` directly, bypasses Auto Layout |
| Android | `FlashFlexLayout` — single measure pass, `onLayout` assigns child frames |
| Web | CSS flex via IR (acceptable; browser owns layout) |

#### Layout IR

Compiler emits constraints at compile time where possible:

```text
NODE Column#0  layout=flex(column) gap=8 padding=16
  NODE Row#1    layout=flex(row) align=center
    NODE Text#2 layout=flexChild(weight=1)
```

Runtime layout engine (Rust, ~2k LOC):

1. Read computed sizes from native text measurement (one-shot per content change)
2. Single top-down flex pass
3. Emit `set_frame` commands (batched in command buffer, not `set_prop`)

#### What stays on platform layout temporarily (Phase 3–4)

- MVP counter app only: `UIStackView` / simple `ViewGroup` is acceptable
- Any screen with >10 nested containers: migrate to `FlashLayoutView`

---

## 3. Revised roadmap

| Phase | Deliverable | Rationale |
|-------|-------------|-----------|
| 1–2 | Compiler + reactive runtime | Core bet |
| 3–4 | iOS/Android primitives (native layout OK for counter) | Prove pipeline |
| **5a** | **Animation IR + platform animators** | Fix worst benchmark before reviewers see it |
| **5b** | **Custom flex layout views** | Fix deep-tree layout before `flights` |
| 5c | List, Image, TextField, ScrollView, navigation | `flights` example |
| 6 | Hot reload, incremental compile, inspector | DX |
| 7 | IR/runtime optimization, benchmarks | Measure against §1–2 targets |
| 8 | GPU renderer investigation | Only if native views are binding constraint |

---

## 4. Decision log additions

| # | Decision | Problem | Runtime cost | Alternative rejected |
|---|----------|---------|--------------|---------------------|
| D21 | Platform-native animations | Per-frame set_prop destroys animation perf | One setup call per animation | Per-frame reactive flush |
| D22 | Compositor-only animation properties | Layout thrash on UIKit/Android | None during animation | Animating width/height |
| D23 | Custom flex layout at Phase 5b | Auto Layout / double-measure loss | One Rust layout pass | UIStackView forever |
| D24 | `set_frame` separate from `set_prop` | Frame changes aren't reactive text updates | Batched in command buffer | Mixing layout into reactive ops |

---

## 5. What we still lose (honest)

- **Text shaping at scale**: Flutter caches aggressively; we delegate to platform until GPU phase.
- **Custom paint / shaders**: Flutter wins; not in scope until Phase 8.
- **Implicit animations on layout**: Until Phase 5b, layout changes are instant or platform-default.

State these in benchmark reports. Do not claim superiority without measuring.

---

## Related docs

- [Flash vs KMP vs Flutter](comparison.md)
- [README](../README.md) — project status and repo layout
