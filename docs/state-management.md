# Flash State Management — `@provider`

**Inspired by Riverpod. Better than Riverpod.**

Riverpod gives you providers, `ref.watch`, and automatic rebuilds. Flash gives
you the same mental model — but dependencies are resolved at **compile time**,
and updates patch **one native property** instead of rebuilding a widget subtree.

No `#[ui_export(writes = "...")]`. No manual dependency lists.

---

## The problem with `writes = "count, high_score"`

```rust
#[ui_export(writes = "count, high_score")]  // ❌ tedious, easy to get wrong
pub fn increment(&mut self) { ... }
```

You are manually telling the compiler what Riverpod figures out automatically.
If you forget a field, UI goes stale. If you over-declare, you waste updates.

**Flash `@provider` replaces this with automatic tracking.**

---

## Core concepts

| Concept | Riverpod | Flash `@provider` |
|---------|----------|-------------------|
| State unit | `Provider` / `Notifier` | `@provider` |
| Reactive read | `ref.watch(provider)` | `${counter.count}` in UI |
| One-shot read | `ref.read(provider)` | read inside `@action` handler body |
| Mutation | `notifier.state = x` | `@action` method or direct `@state` assign |
| Async state | `AsyncValue<T>` / `AsyncNotifier` | `@async` + `match` in `.ui` |
| Scoped lifetime | `ProviderScope` / overrides | `@scope(App \| Screen \| Family)` |
| Dependency tracking | **Runtime** (ref container) | **Compile time** (static graph) |
| UI update | Rebuild widget subtree | `set_prop` on affected nodes only |

---

## Pattern 1 — `.ui` provider (simple state)

For counters, toggles, form fields — no Rust needed.

```ui
@provider Counter {
    @state count: Int = 0
    @state high_score: Int = 0

    @action increment() {
        count++
        if count > high_score {
            high_score = count
        }
    }

    @action reset() {
        count = 0
    }
}

screen Home {
    @inject counter: Counter

    Column {
        Text("Count: ${counter.count}")
        Text("Best: ${counter.high_score}")

        Row {
            Button("−") { counter.count-- }
            Button("+") { counter.increment() }
            Button("Reset") { counter.reset() }
        }
    }
}
```

**What the compiler does:**

```text
counter.count      → Text#1.text     (auto-watch from ${})
counter.high_score → Text#2.text     (auto-watch from ${})
counter.increment  → writes [count, high_score]  (inferred from @action body)
counter.reset      → writes [count]                (inferred — high_score untouched)
```

Same fine-grained behavior as today's `count++` counter — zero annotations.

---

## Pattern 2 — Rust provider (business logic)

When you need HTTP, validation, or complex types — implement in Rust.
The proc macro tracks `@state` fields; **no `writes` attribute**.

```rust
use flash_provider::{provider, state, action, async_action};

#[provider]
pub struct Counter {
    #[state] count: i64,
    #[state] high_score: i64,
}

#[provider]
impl Counter {
    #[action]
    pub fn increment(&mut self) {
        self.count += 1;
        if self.count > self.high_score {
            self.high_score = self.count;
        }
    }

    #[action]
    pub fn reset(&mut self) {
        self.count = 0;
    }
}
```

```ui
use presentation::Counter

screen Home {
    @inject counter: Counter

    Column {
        Text("Count: ${counter.count}")
        Text("Best: ${counter.high_score}")
        Button("Increment") { counter.increment() }
    }
}
```

The `#[provider]` macro:
1. Emits `.uiapi` manifest (types, methods, state slots)
2. **Infers writes** by analyzing assignments to `#[state]` fields in each `#[action]`
3. Falls back to `@mutates(count, high_score)` in `.uiapi` only when inference fails (indirect mutation)

---

## Pattern 3 — Async provider (like Riverpod `AsyncNotifier`)

```rust
#[provider]
pub struct Flights {
    #[state] value: AsyncValue<List<FlightRow>>,
    search: SearchFlightsUseCase,  // not @state — injected dependency
}

#[provider]
impl Flights {
    #[async_action]
    pub async fn load(&mut self, query: String) {
        self.value = AsyncValue::loading();
        match self.search.execute(&query).await {
            Ok(rows) if rows.is_empty() => self.value = AsyncValue::empty(),
            Ok(rows)  => self.value = AsyncValue::data(rows),
            Err(e)    => self.value = AsyncValue::error(e.message()),
        }
    }
}
```

```ui
@inject flights: Flights

screen Flights(query: String) {
    onLoad { await flights.load(query) }

    match flights.value {
        Loading  -> Column { Loading(); Text("Searching...") }
        Empty    -> Text("No flights found")
        Error(e) -> Column { Text(e); Button("Retry") { await flights.load(query) } }
        Data(rows) -> List(rows) key: row.id { row in FlightCard(row) }
    }
}
```

`match flights.value` compiles to a **region** — only the active arm's nodes
exist. State change `Loading → Data` swaps the region, not the whole screen.

---

## Pattern 4 — Scoped providers (like Riverpod scopes)

```ui
@provider(scope: App)
AuthSession {
    @state user: User? = none
    @state is_logged_in: Bool = false

    @action login(token: String) { ... }
    @action logout() { user = none; is_logged_in = false }
}

@provider(scope: Screen)
CheckoutForm {
    @state card_number: String = ""
    @state expiry: String = ""
}

screen Profile {
    @inject auth: AuthSession       // same instance app-wide
    Text("Hello ${auth.user?.name}")
}

screen Checkout {
    @inject auth: AuthSession       // same App-scoped instance
    @inject form: CheckoutForm       // new instance per screen visit
}
```

| Scope | Lifetime | Riverpod equivalent |
|-------|----------|---------------------|
| `@scope(App)` | Process lifetime | Root provider |
| `@scope(Screen)` | While screen mounted | Auto-dispose provider |
| `@scope(Family)` | Keyed by parameter | `.family` modifier |

---

## Pattern 5 — Parameterized providers (like Riverpod `.family`)

```ui
@provider(scope: Family, key: userId)
UserProfile(userId: String) {
    @state user: User? = none

    @async_action load() {
        user = await api.fetch_user(userId)
    }
}

screen UserDetail(userId: String) {
    @inject profile: UserProfile(userId)

    onLoad { await profile.load() }

    if profile.user == none {
        Loading()
    } else {
        Text("${profile.user.name}")
    }
}
```

Each `userId` gets its own cached provider instance — like `userProfileProvider(userId)`.

---

## How auto-watch works (vs Riverpod `ref.watch`)

### Riverpod

```dart
final count = ref.watch(counterProvider);  // explicit watch
return Text('$count');                   // rebuilds this widget
```

### Flash

```ui
Text("Count: ${counter.count}")   // compiler registers: counter.count → Text#1
```

Any `${provider.field}` or `match provider.field` in UI **automatically registers
a compile-time dependency edge**. No `ref`, no `watch()` call, no `BuildContext`.

Inside `@action` / handler bodies, reads are **one-shot** (like `ref.read`) —
they do not register UI dependencies.

---

## How auto-invalidation works (vs `notifyListeners`)

### Riverpod

```dart
state = state.copyWith(count: state.count + 1);
notifyListeners();  // runtime: find all watchers → rebuild them
```

### Flash

```ui
@action increment() {
    count++   // compiler knows: increment writes [count]
              // runtime: mark slot dirty → flush → set_prop(Text#1)
}
```

For Rust `#[action]` methods, the proc macro scans for assignments to `#[state]`
fields:

```rust
#[action]
pub fn increment(&mut self) {
    self.count += 1;                    // → writes count
    if self.count > self.high_score {
        self.high_score = self.count;   // → writes high_score
    }
}
// emitted .uiapi: increment @mutates(count, high_score)  — automatic
```

---

## Side effects — `@listen` (like Riverpod `ref.listen`)

```ui
screen Home {
    @inject counter: Counter

    @listen(counter.high_score) { new_score in
        if new_score > 10 {
            haptic_heavy()
        }
    }

    Text("Best: ${counter.high_score}")
}
```

`@listen` runs a side effect when a field changes — does not affect UI graph.

---

## Comparison: counter in each framework

### React Native + Zustand

```jsx
const useCounter = create(set => ({
  count: 0,
  increment: () => set(s => ({ count: s.count + 1 })),
}));

function Home() {
  const { count, increment } = useCounter();
  return <Button onPress={increment}><Text>{count}</Text></Button>;
}
```

Runtime store subscription → re-render component → reconcile.

### Riverpod

```dart
@riverpod
class Counter extends _$Counter {
  @override int build() => 0;
  void increment() => state++;
}

// UI
final count = ref.watch(counterProvider);
Text('$count');
```

Runtime provider container → mark dependents dirty → rebuild widgets.

### Flash `@provider`

```ui
@provider Counter {
    @state count: Int = 0
    @action increment() { count++ }
}

screen Home {
    @inject counter: Counter
    Text("${counter.count}")
    Button("+") { counter.increment() }
}
```

Compile-time: `counter.count → Text#1`, `increment → writes [count]`.
Runtime: one `set_prop` per tap. No rebuild.

---

## Migration from old `#[ui_export(writes)]` model

| Old | New |
|-----|-----|
| `#[ui_export]` struct | `#[provider]` struct |
| `#[ui_export(writes = "x")]` | `#[action]` + `#[state]` (auto-inferred) |
| `screen Home(vm: VM)` | `screen Home { @inject vm: VM }` |
| `state count: Int = 0` in screen | `@provider` with `@state count` |
| `vm.load()` + manual writes | `#[async_action]` + `AsyncValue` |

Screen-local `state` still works for trivial cases — it is sugar for an
anonymous `@provider(scope: Screen)`.

---

## Why this is better than Riverpod

| | Riverpod | Flash `@provider` |
|--|----------|-------------------|
| Dependency tracking | Runtime (`ref.watch` registry) | Compile time (static CSR graph) |
| Update granularity | Rebuild widget subtree | Patch single native property |
| `ref` / `BuildContext` | Required everywhere | Not needed — `@inject` on screen |
| Async state | `AsyncValue` ✅ | `AsyncValue` + `match` in `.ui` ✅ |
| Family / scope | ✅ | `@scope` + `@family` ✅ |
| Code generation | `@riverpod` macro | `#[provider]` macro |
| GC / runtime overhead | Dart VM | Rust, no GC |
| Wrong `writes` annotation | N/A (auto) | N/A (auto-inferred) |
| Hot reload | ✅ | `.ui` providers ✅; Rust needs rebuild |

Riverpod's DX with compile-time guarantees and native-widget performance.

---

## Implementation roadmap

| Feature | Phase |
|---------|-------|
| `@provider` / `@state` / `@action` in `.ui` | Phase 5a |
| Auto-watch from `${}` interpolation | Phase 5a (exists for screen `state`) |
| `#[provider]` / `#[state]` / `#[action]` Rust macro | Phase 5b |
| Auto-infer writes from `#[action]` body | Phase 5b |
| `@async` / `AsyncValue` / region `match` | Phase 5c |
| `@scope` / `@inject` / `@family` | Phase 5c |
| `@listen` side effects | Phase 6 |

Phase 1–2 screen `state` is the simplest form of `@provider(scope: Screen)`.
