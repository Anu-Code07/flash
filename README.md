# Flash

A compiled cross-platform UI framework for **iOS, Android, and Web**.

Declarative `.ui` source is transformed at compile time into a flat UI IR plus a
static reactive dependency graph, executed by a minimal Rust runtime that issues
batched native platform UI operations.

## Status

Phase 1–3 in progress: compiler pipeline + reactive runtime + native host bridge.
Counter example compiles to IR, updates one native property on `count++`, and renders
via UIKit (iOS) / Android Views through a batched command buffer.

### Install (Flutter/RN-style — one command)

```bash
curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash
source ~/.zshrc
flash doctor
flash create my_app
cd my_app && flash run
```

Installs Rust (if needed), clones SDK to `~/.flash/sdk`, puts `flash` on PATH.

From a git clone: `./scripts/install-flash.sh` or `cargo install --path cli --force`

Without a global install: `cargo run -p flash-cli -- create my_app`

### Commands

```bash
cargo run -p flash-cli -- ir examples/counter/home.ui
cargo run -p flash-cli -- run examples/counter/home.ui
cargo run -p flash-cli -- run --native examples/counter/home.ui  # native command buffer
cargo run -p flash-cli -- run ios examples/counter/home.ui       # UIKit path
cargo run -p flash-cli -- dev examples/counter/home.ui           # hot reload on save
cargo run -p flash-cli -- dev --native examples/counter/home.ui  # native hot reload
cargo run -p flash-cli -- create MyApp all                       # scaffold iOS + Android
cargo run -p flash-cli -- build ios                              # native build guide
```

### Deploy docs site (Vercel)

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2FAnu-Code07%2Fflash&project-name=flash-docs&repository-name=flash)

One-click: import the repo on Vercel — `vercel.json` is already configured (`outputDirectory: site`).

## Language documentation site

Browse the full language reference, STL, examples, and mobile guide:

```bash
cargo run -p flash-cli -- docs
# Open http://localhost:3000
```

Or open `site/index.html` directly. Pages include:

- **Language** — syntax, state, handlers, async, match
- **STL** — collections, strings, math, mobile APIs, 30+ modifiers, design tokens
- **Components** — built-in UI primitives with iOS/Android mapping
- **Examples** — counter, flights, animation, navigation tabs
- **Extensions** — custom modifiers, view models, platform hooks
- **Mobile** — safe areas, haptics, platform detection, keyboard avoidance

## Design docs

- [State management — `@provider` (Riverpod-style, compile-time)](docs/state-management.md)
- [Developer guide — UI + business logic](docs/developer-guide.md)
- [Flash vs KMP vs Flutter](docs/comparison.md)
- [Animation and layout — known weaknesses and mitigations](docs/animation-and-layout.md)
- [Widget comparison — Flash vs Flutter vs React Native](docs/widgets-comparison.md)

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
examples/     Counter, flights, animation, navigation examples
site/         Language documentation website (flash docs)
```
