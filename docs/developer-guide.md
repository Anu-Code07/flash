# Flash Developer Guide — UI + Business Logic

How an end user builds a real app (counter, data fetching, lists) with Flash.

---

## The three-tier model

Flash splits code into three layers. **End users write two of them** (`.ui` + Rust).
Platform Swift/Kotlin is only for SDK escape hatches.

| Tier | Language | Who writes it | Owns |
|------|----------|---------------|------|
| **0** | `.ui` | App developer | Layout, bindings, screen-local state, `if`/`match`, lists |
| **1** | **Rust** | App developer | Use cases, repositories, view models, HTTP, validation |
| **2** | Swift / Kotlin | Rare | Camera, HealthKit, payment SDKs |

**Rule of thumb:** if it needs a loop body, a type definition, or more than one `await`,
it belongs in **Rust**, not `.ui`.

---

## What goes where

### In `.ui` (Tier 0)

```text
state count: Int = 0              // screen-local UI state
if loading { ... }              // conditional UI
List(flights) { f in ... }      // iteration over data
Text("${vm.title}")             // read from Rust view model
onLoad { vm.load() }            // call into Rust
Button("Retry") { vm.load() }   // event → Rust method
```

### In Rust (Tier 1)

```rust
// entities, use cases, HTTP, mapping, error handling
pub struct FlightsViewModel { ... }

impl FlightsViewModel {
    pub async fn load(&mut self) { ... }   // fetches from API
    pub fn state(&self) -> &FlightsState { ... }
}
```

### In Swift/Kotlin (Tier 2) — only when needed

```swift
MyUI.register("HealthKitChart") { props in ... }
```

---

## Project structure (what a real app looks like)

```text
my_counter_app/
├── ui/                          ← Tier 0: screens & components
│   ├── screens/
│   │   ├── home.ui
│   │   └── flights.ui
│   └── components/
│       └── flight_card.ui
│
├── crates/
│   ├── domain/                  ← entities, business rules (no IO)
│   ├── application/             ← use cases + port traits
│   ├── infrastructure/          ← HTTP, DB, platform adapters
│   ├── presentation/            ← view models exposed to .ui
│   └── app/                     ← wires everything together
│
├── platform/
│   ├── ios/                     ← Xcode project, UIKit host
│   └── android/                 ← Gradle project, View host
│
└── flash.toml                   ← app config (routes, deps)
```

**Dependency rule:** `.ui` may only `use` from `presentation`. It cannot import
`infrastructure` directly — that is a compile error (`UI1008`).

---

## Example 1: Simple counter (logic stays in `.ui`)

For trivial UI-only state, no Rust view model is needed.

**`ui/screens/home.ui`**

```text
screen Home {
    state count: Int = 0

    Column {
        Text("Count: ${count}")

        Button("Increment") {
            count++
        }

        Button("Decrement") {
            count--
        }
    }
}
```

**What happens at compile time:**

```
count → Text#1.text          (dependency edge)
count++ → Handler writes slot 0
```

**What happens at runtime:**

```
tap → count++ → mark dirty → flush → set_prop(Text, "Count: 1")
```

No Rust business logic. This works today (Phase 1–2).

---

## Example 2: Counter with saved high score (needs Rust)

When logic grows beyond `count++`, move it to a Rust view model.

**`crates/presentation/src/counter_vm.rs`**

```rust
use flash_export::ui_export;

#[ui_export]
pub struct CounterViewModel {
    count: i64,
    high_score: i64,
}

#[ui_export]
impl CounterViewModel {
    pub fn count(&self) -> i64 { self.count }
    pub fn high_score(&self) -> i64 { self.high_score }

    #[ui_export(writes = "count, high_score")]
    pub fn increment(&mut self) {
        self.count += 1;
        if self.count > self.high_score {
            self.high_score = self.count;
        }
    }

    #[ui_export(writes = "count")]
    pub fn reset(&mut self) {
        self.count = 0;
    }
}
```

The `#[ui_export]` macro emits `presentation.uiapi` — a manifest the `.ui`
compiler reads so it knows types, methods, and which state slots each method writes.

**`ui/screens/home.ui`**

```text
use presentation::CounterViewModel

screen Home(vm: CounterViewModel) {

    Column {
        Text("Count: ${vm.count}")
        Text("High score: ${vm.high_score}")

        Row {
            Button("Increment") { vm.increment() }
            Button("Reset")       { vm.reset() }
        }
    }
}
```

**`crates/app/src/main.rs`** (composition root — wires dependencies)

```rust
pub fn build_app() -> App {
    App {
        home_vm: CounterViewModel { count: 0, high_score: 0 },
    }
}
```

The screen receives `vm` as a **parameter** — no service locator, no globals.
Easy to test with a fake view model.

---

## Example 3: Flights with data fetching (full pattern)

This is the pattern for real apps: async fetch, loading/error states, lists.

### Step 1 — Domain (pure data)

**`crates/domain/src/flight.rs`**

```rust
pub struct Flight {
    pub id: String,
    pub airline: String,
    pub fare_cents: i64,
}
```

### Step 2 — Application (use case + port)

**`crates/application/src/ports.rs`**

```rust
#[async_trait(?Send)]
pub trait FlightRepository {
    async fn search(&self, query: &str) -> Result<Vec<Flight>, AppError>;
}
```

**`crates/application/src/search_flights.rs`**

```rust
pub struct SearchFlights<R: FlightRepository> {
    repo: R,
}

impl<R: FlightRepository> SearchFlights<R> {
    pub async fn execute(&self, query: &str) -> Result<Vec<Flight>, AppError> {
        self.repo.search(query).await
    }
}
```

### Step 3 — Infrastructure (HTTP adapter)

**`crates/infrastructure/src/http_flights.rs`**

```rust
pub struct HttpFlightRepository {
    http: PlatformHttp,   // URLSession on iOS, OkHttp on Android
}

#[async_trait(?Send)]
impl FlightRepository for HttpFlightRepository {
    async fn search(&self, query: &str) -> Result<Vec<Flight>, AppError> {
        let response = self.http.get("/flights", query).await?;
        Ok(response.into_flights())
    }
}
```

HTTP goes through a **platform port** — not `reqwest` — so TLS, proxy, and
connection pooling use the OS stack.

### Step 4 — Presentation (view model exposed to `.ui`)

**`crates/presentation/src/flights_vm.rs`**

```rust
#[ui_export]
pub enum FlightsState {
    Loading,
    Loaded(Vec<FlightRow>),
    Empty,
    Error(String),
}

#[ui_export]
pub struct FlightRow {
    pub id: String,
    pub airline: String,
    pub fare: String,       // already formatted for display
}

#[ui_export]
pub struct FlightsViewModel<UC> {
    search: UC,
    state: FlightsState,
}

#[ui_export]
impl<UC: SearchFlightsPort> FlightsViewModel<UC> {
    pub fn state(&self) -> &FlightsState { &self.state }

    #[ui_export(writes = "state")]
    pub async fn load(&mut self, query: String) {
        self.state = FlightsState::Loading;
        match self.search.execute(&query).await {
            Ok(flights) if flights.is_empty() => {
                self.state = FlightsState::Empty;
            }
            Ok(flights) => {
                self.state = FlightsState::Loaded(flights.into_rows());
            }
            Err(e) => {
                self.state = FlightsState::Error(e.user_message());
            }
        }
    }
}
```

### Step 5 — UI (`.ui` screen)

**`ui/screens/flights.ui`**

```text
use presentation::{FlightsViewModel, FlightsState, FlightRow}

screen Flights(vm: FlightsViewModel, query: String) {

    onLoad {
        vm.load(query)
    }

    match vm.state {
        Loading -> Loading()

        Empty -> Text("No flights found")

        Error(message) -> Column {
            Text(message)
            Button("Retry") { vm.load(query) }
        }

        Loaded(rows) -> List(rows) key: row.id { row in
            FlightCard(
                airline: row.airline,
                fare: row.fare
            )
        }
    }
}
```

**`ui/components/flight_card.ui`**

```text
component FlightCard(airline: String, fare: String) {
    Row {
        Text(airline)
        Text(fare)
    }
}
```

### What the compiler does

```
1. Reads presentation.uiapi (from #[ui_export])
2. Type-checks vm.load(query), match vm.state arms
3. Builds dependency edges:
     vm.state → all nodes inside match arms (via region)
4. vm.load writes state → invalidates region → swaps Loading/Loaded/Error variant
5. List(rows) uses keyed reconciliation for row updates
```

### What the runtime does on `vm.load()` tap

```
1. Handler calls generated async Rust fn
2. vm.state = Loading  →  region re-renders Loading spinner
3. await http.get(...)  (suspends on UI thread executor)
4. resume: vm.state = Loaded(rows)  →  region swaps to List
5. Only the region's nodes are created/destroyed — not the whole screen
```

---

## Build flow (what `flash build` does)

```text
1. cargo build -p presentation
       ↓ #[ui_export] writes target/presentation.uiapi

2. flash compile ui/
       ↓ reads .uiapi, emits generated Rust into OUT_DIR

3. cargo build -p app
       ↓ links generated UI + view models + platform adapter

4. Platform build
       ↓ iOS: static lib → Xcode
       ↓ Android: .so → Gradle
```

One command hides this:

```bash
flash dev          # watch .ui + incremental compile + hot reload
flash run ios      # build + install + launch
flash run android
```

---

## Developer workflow summary

| Task | Where | Language |
|------|-------|----------|
| Layout, buttons, lists | `ui/*.ui` | `.ui` |
| Screen-local toggle state | `state x: Bool = false` in `.ui` | `.ui` |
| API calls, validation, mapping | `crates/application/` | Rust |
| HTTP, DB, SDK adapters | `crates/infrastructure/` | Rust |
| What `.ui` can see | `crates/presentation/` | Rust + `#[ui_export]` |
| Wire dependencies | `crates/app/` | Rust |
| Camera, HealthKit, etc. | `platform/ios`, `platform/android` | Swift/Kotlin |

---

## What is implemented today vs coming

| Feature | Status |
|---------|--------|
| `.ui` screens, state, components | ✅ Phase 1–2 |
| Reactive dependency graph | ✅ Phase 1–2 |
| `use presentation::...` imports | 🔜 Phase 5 (`#[ui_export]` + `.uiapi`) |
| `onLoad { await vm.load() }` | 🔜 Phase 5 |
| `match` on Rust enums | 🔜 Phase 5 |
| `List` with keyed reconciliation | 🔜 Phase 5c |
| HTTP via platform port | 🔜 Phase 5 |
| `flash dev` hot reload | 🔜 Phase 6 |
| iOS/Android native hosts | 🔜 Phase 3–4 |

**Today** you can build the simple counter entirely in `.ui`.
**Phase 5** unlocks the full flights pattern with Rust view models and async fetch.

---

## Mental model for app developers

Think of Flash like this:

```
Flutter:  everything in Dart (UI + logic)
KMP:      UI in Compose, logic in Kotlin
Flash:    UI in .ui,   logic in Rust
```

You are **not** writing Rust inside `.ui`. You are **not** calling repositories
from `.ui`. You write a thin view model in Rust, export it with `#[ui_export]`,
and bind to it from `.ui` like SwiftUI binds to an `@Observable` object — except
the binding graph is computed at compile time, not discovered at runtime.

---

## Related docs

- [Flash vs KMP vs Flutter](comparison.md)
- [Animation and layout mitigations](animation-and-layout.md)
