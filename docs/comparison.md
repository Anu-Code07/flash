# Flash vs KMP vs Flutter

A practical comparison for teams evaluating cross-platform mobile (iOS, Android)
and web UI approaches.

**Last updated:** 2026-09-15

---

## One-line summary

| Framework | What it is |
|-----------|------------|
| **Flash** | Compiled `.ui` → static UI IR → native widgets, Rust logic, minimal runtime |
| **KMP** | Shared Kotlin logic; UI is usually Compose Multiplatform or fully separate per platform |
| **Flutter** | Dart UI + widget tree + own renderer (Skia/Impeller), full framework runtime |

---

## Architecture at a glance

```
Flash
  .ui source → compiler → UI IR + dependency table
       ↓
  Rust runtime (slots, flush) → command buffer → UIKit / Android View / DOM

KMP (typical: Compose Multiplatform)
  Kotlin @Composable → Compose runtime → recomposition → Skia draw

KMP (logic-only pattern)
  Kotlin shared logic ←→ SwiftUI (iOS) + Jetpack Compose (Android)  [no shared UI]

Flutter
  Dart widgets → build() → element tree → diff → Skia/Impeller
```

---

## Side-by-side

| Dimension | Flash | KMP (Compose) | KMP (logic only) | Flutter |
|-----------|-------|---------------|------------------|---------|
| **Shared UI** | Yes (`.ui`) | Yes (Compose) | No | Yes (Dart widgets) |
| **Shared logic language** | Rust | Kotlin | Kotlin | Dart |
| **UI update model** | Compile-time dependency graph; slot → update ops | Runtime recomposition | Platform-native (varies) | Runtime rebuild + diff |
| **Virtual DOM / reconciler** | No | No (Compose slot table) | Platform-dependent | Element tree diff |
| **Renderer** | Native widgets (Phase 1–7) | Compose/Skia | Platform widgets | Own engine |
| **JS runtime** | No | No | No | No |
| **GC shipped** | No (Rust) | Yes (Kotlin) | Yes (Kotlin) | Yes (Dart) |
| **Hot reload** | `.ui` only (Phase 6); Rust needs rebuild | Compose + Kotlin | Platform tools | Dart + widgets |
| **Web story** | WASM + DOM (planned) | Compose HTML / JS | Separate web stack | Web from day one |
| **Maturity** | Pre-1.0 | Production (Compose MP growing) | Production (logic) | Production |
| **Hiring / onboarding** | Rust + custom `.ui` | Kotlin (large pool) | Kotlin + 2 UI stacks | Dart (moderate pool) |

---

## Where they overlap

All four approaches (including KMP logic-only) share some goals:

- Ship iOS and Android from one codebase (web varies)
- Avoid JavaScript bridges for UI
- Share business logic across platforms
- Platform escape hatches for SDKs (camera, payments, biometrics)

That is the extent of similarity between **Flash** and **KMP**. The UI engines are
not comparable.

---

## Flash vs KMP

### KMP is not one thing

Kotlin Multiplatform usually means one of two patterns:

**Pattern A — Compose Multiplatform (most common for “KMP UI”)**

```
@Composable fun Screen() { ... }
        ↓
Compose runtime (recomposition)
        ↓
Skia / drawing layer
```

This is architecturally closer to **Flutter** than to Flash. Both own the render
pipeline and pay a framework runtime cost on every frame.

**Pattern B — Logic only**

```
shared/src/commonMain/   ← Kotlin use cases, repos, models
iosApp/                  ← SwiftUI
androidApp/              ← Jetpack Compose or Views
```

No shared UI. Flash’s three-tier model is closer to this *structurally*:

| Tier | Flash | KMP logic-only |
|------|-------|----------------|
| UI | `.ui` | SwiftUI / Compose (per platform) |
| Logic | Rust | Kotlin |
| Platform | Swift / Kotlin SDKs | Swift / Kotlin SDKs |

But Flash still differs:

1. **UI is compiled, not platform-native syntax** — one `.ui` file, not two UI codebases
2. **Reactivity is static** — compiler emits `count → Text#1`; Compose recomposes subtrees at runtime
3. **No Kotlin runtime on device** for UI path — Rust AOT + thin native adapter

### When KMP is the better choice

- Team already strong in Kotlin
- Want Compose Multiplatform’s widget catalogue and ecosystem today
- Accept GC + Compose runtime for faster time-to-market
- JetBrains tooling and community matter

### When Flash is the better choice

- Minimizing runtime overhead and binary size is a primary goal
- Fine-grained updates without recomposition matter (feeds, counters, real-time UI)
- Team accepts Rust for logic and a small custom UI language
- Native look-and-feel via real `UILabel` / `TextView` is required from v1

### KMP pain points Flash avoids (and trades for others)

| KMP issue | Flash response |
|-----------|----------------|
| Kotlin/Native iOS memory model / interop rough edges | Rust → C FFI / JNI; no Kotlin on iOS |
| Compose MP still maturing on iOS | Native widgets from Phase 3 |
| GC pauses | No GC in UI hot path |
| **Flash pays:** smaller ecosystem, unproven `.ui` language, Rust hiring friction |

---

## Flash vs Flutter

| Dimension | Flash | Flutter |
|-----------|-------|---------|
| **Widget model** | Flat IR node array | Deep widget tree + `build()` |
| **Updates** | Precomputed slot → op table | Rebuild subtree + diff elements |
| **Rendering** | Native widgets (v1) | Skia/Impeller always |
| **Language** | `.ui` + Rust | Dart everywhere |
| **Accessibility** | Free from native views | Must implement in engine |
| **Custom paint** | Deferred (Phase 8) | Core strength |
| **Animation** | Must use platform animators (Phase 5a) | Engine-owned, raster thread |
| **Layout** | Platform layout until Phase 5b | Single-pass in engine |

Flutter wins on: animation, layout consistency, custom drawing, ecosystem, maturity.

Flash wins on (if benchmarks confirm): native widget fidelity, potentially smaller
runtime, no GC, discrete state-update latency without tree walks.

---

## UI update model (the core difference)

### Flash

```text
state count: Int = 0
Text("Count: ${count}")

Compiler emits:
  count → [UpdateOp Text#1.text]

Runtime on count++:
  1. slot[0] += 1
  2. lookup deps[0] → [op0]
  3. set_prop(Text#1, "Count: 1")
  4. commit()  — one FFI batch
```

No `build()`. No tree walk. No diff.

### Flutter

```text
Text('Count: $count')  inside build()

On setState:
  1. mark element dirty
  2. call build() on subtree
  3. diff new widget vs old element
  4. update render object
```

### Compose (KMP)

```text
@Composable fun Counter() {
    Text("Count: $count")
}

On count change:
  1. recomposition scope invalidated
  2. recompose affected composables
  3. Compose runtime applies changes to slot table
  4. draw via Skia
```

Flash moves reconciliation **to compile time**. Flutter and Compose do it **at
runtime**. That is the bet.

---

## Decision guide

```
Do you need production-ready UI today?
  └─ Yes → Flutter or KMP + Compose

Is the team Kotlin-first and happy with Compose runtime?
  └─ Yes → KMP (Compose Multiplatform)

Do you need native widgets (UIKit/TextView) without a custom renderer?
  └─ Yes → Flash or KMP logic-only (but then two UI codebases)

Is minimal runtime + compile-time reactivity the top priority?
  └─ Yes → Flash (accept pre-1.0 risk)

Is Rust acceptable for shared logic?
  └─ No  → KMP logic-only or Flutter
  └─ Yes → Flash
```

---

## Honest scorecard (pre-benchmark)

| Metric | Flash (target) | KMP Compose | Flutter |
|--------|----------------|-------------|---------|
| Time to first screen | Unknown | Good | Good |
| Discrete state update | Best (if thesis holds) | Good | Good |
| Animation (1000 nodes) | Tie (with Phase 5a) | Good | Best |
| Deep layout trees | Weak until Phase 5b | Good | Best |
| Native a11y | Best (native views) | Good (improving) | Must build |
| Binary size (hello world) | Target: small | Medium | Larger |
| Ecosystem | None | Growing | Large |
| Team velocity | Low (new stack) | High (Kotlin) | High (Dart) |

Do not cite this table in marketing until `benchmarks/` has real numbers on
physical devices (see `docs/animation-and-layout.md` §28).

---

## Related docs

- [Animation and layout mitigations](animation-and-layout.md) — where Flash loses to Flutter and how to fix it
- [README](../README.md) — project status and repo layout
