# Flash

A compiled cross-platform UI framework for **iOS, Android, and Web**.

Declarative `.ui` source is transformed at compile time into a flat UI IR plus a
static reactive dependency graph, executed by a minimal Rust runtime that issues
batched native platform UI operations.

## Status

Phase 1–2 complete: compiler pipeline + reactive runtime. Counter example compiles
to IR and updates only the dependent `Text` node on `count++`.

```bash
cargo run -p flash-cli -- ir examples/counter/home.ui
cargo run -p flash-cli -- run examples/counter/home.ui
```

## Design docs

- [Flash vs KMP vs Flutter](docs/comparison.md)
- [Animation and layout — known weaknesses and mitigations](docs/animation-and-layout.md)

Key points from that doc:

1. **Animation (worst metric):** Never drive animations through per-frame reactive
   flush. Compile `.withAnimation` into Core Animation / Android Animator — animate
   `opacity` and `transform` only, zero crossings during the animation.

2. **Layout (second weakness):** Custom flex layout moves to **Phase 5b** (before
   `flights`), not Phase 8. `FlashLayoutView` / `FlashFlexLayout` bypass Auto Layout
   and double-measure.

## Repository layout

```
compiler/     Lexer → parser → sema → UI IR
runtime/      State, reactive flush, MockRenderer
platform/     iOS, Android, Web adapters
stl/          Standard types (C++ STL-like)
docs/         Architecture and mitigation plans
examples/     Counter and future flights example
```
