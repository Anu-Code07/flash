# Flash-Exclusive Widgets

Widgets that only exist in Flash — built for real mobile pain points that Flutter
and React Native solve with packages, boilerplate, or runtime rebuilds.

Flash widgets compile to **fine-grained native updates** via `@provider` binding.
No widget subtree rebuilds. No manual dependency lists.

---

## Why Flash-exclusive widgets?

| Problem | Flutter / RN approach | Flash widget |
|---------|----------------------|--------------|
| Async loading UI | `FutureBuilder` / manual `useState` + `useEffect` | `AsyncView` |
| OTP input | Custom `TextField` + focus logic | `PinField` |
| Swipe to delete | `Dismissible` / `react-native-swipe-list-view` | `SwipeActionRow` |
| Pull to refresh | `RefreshIndicator` + manual callback | `PullToRefresh` bound to `@action` |
| Empty list state | Copy-paste Column every screen | `EmptyState` |
| Error + retry | Copy-paste every screen | `ErrorState` |
| Bottom checkout bar | `Scaffold.bottomNavigationBar` + keyboard hacks | `BottomCTA` |
| Offline detection | `connectivity_plus` package | `OfflineBanner` |
| Screenshot blocking | Platform channels / plugins | `SecureScreen` |
| Haptic buttons | Manual `HapticFeedback.lightImpact()` | `HapticButton` |

---

## Async & reactive state

### `AsyncView` — replace manual `match` blocks

**Use case:** Every screen that loads data from an API.

Instead of writing Loading / Empty / Error / Data branches manually:

```ui
// Before — 20+ lines per screen
match flights.value {
    Loading -> Column { Loading(); Text("Searching...") }
    Empty   -> Text("No flights found")
    Error(msg) -> Column { Text(msg); Button("Retry") { await flights.load() } }
    Data(rows) -> List(rows) key: row.id { row in FlightCard(...) }
}

// After — one widget, compile-time slots
AsyncView(flights.value) {
    loading -> SkeletonList(rows: 5)
    empty   -> EmptyState("No flights", "Try a different date")
    error   -> ErrorState { await flights.load(query) }
    data    -> List(rows) key: row.id { row in FlightCard(...) }
}
```

The compiler wires each slot to the correct `AsyncValue` variant and patches
only the affected native nodes on state change.

### `BoundText` — auto-watch provider fields

**Use case:** Displaying live counters, prices, cart totals without `${}` syntax.

```ui
@inject cart: Cart
BoundText(cart.total)   // auto-watches cart.total, fine-grained Text update
```

### `ProviderScope` — inject provider into subtree

**Use case:** Passing a provider to a deep widget tree without prop drilling.

```ui
ProviderScope(checkout: Checkout) {
    CheckoutHeader()
    CheckoutItems()
    BottomCTA("Pay ${checkout.total}") { checkout.pay() }
}
```

### `ListenEffect` — side effects on state change

**Use case:** Haptic on error, analytics on success, navigation on auth change.

```ui
ListenEffect(auth.isLoggedIn) {
    if !auth.isLoggedIn { navigate("/login") }
}
```

---

## Screen patterns

### `EmptyState` — every list/search screen

```ui
EmptyState(
    icon: "airplane",
    title: "No flights found",
    message: "Try different dates or airports"
) {
    Button("Search again") { await flights.load(query) }
}
```

### `ErrorState` — standard error recovery

```ui
ErrorState("Connection failed") {
    await flights.load(query)   // wired to retry @action
}
```

### `BottomCTA` — checkout, onboarding, forms

```ui
Column {
    ScrollView { /* form fields */ }
    BottomCTA("Continue") {
        if form.isValid { navigate("/payment") }
    }
}
```

Floats above keyboard automatically via `KeyboardSafe` integration.

---

## Forms & validation

### `PinField` — OTP / PIN verification

**Use case:** Banking apps, 2FA, phone verification.

```ui
PinField(length: 6) {
    auth.submitOtp(value)
}
```

Auto-advance between digits, secure masking, paste support, countdown via `CountdownLabel`.

### `PhoneField` — international phone input

```ui
PhoneField {
    signup.setPhone(value)
}
```

Country picker, E.164 formatting, validation bound to `@provider`.

### `CurrencyField` — money input

```ui
CurrencyField(currency: "USD", max: 9999.99) {
    payment.setAmount(value)
}
```

### `FormField` — label + input + error

```ui
FormField("Email", error: signup.emailError) {
    ValidatedInput { signup.setEmail(value) }
}
```

---

## Lists & loading

### `SkeletonList` — no layout shift while loading

```ui
if flights.isLoading {
    SkeletonList(rows: 8)
} else {
    List(flights.data) key: f.id { f in FlightCard(...) }
}
```

### `InfiniteScroll` — paginated feeds

```ui
InfiniteScroll(feed.items, onEnd: feed.loadMore) key: item.id { item in
    PostCard(item)
}
```

### `SwipeActionRow` — mail, todo, chat apps

```ui
SwipeActionRow("Archive", "Delete") {
    ListTile(title: email.subject)
} on_delete {
    inbox.delete(email.id)
}
```

---

## Platform & security

### `OfflineBanner` — network status

Auto-shows when connectivity drops. No `connectivity_plus` package.

### `BiometricGate` — banking / wallet unlock

```ui
BiometricGate {
    WalletBalance()
    TransactionList()
}
```

### `SecureScreen` — block screenshots

```ui
SecureScreen {
    AccountNumber(value: account.number)
    CopyableText(account.routing)
}
```

### `PermissionGate` — camera, location, notifications

```ui
PermissionGate(.camera, rationale: "Scan QR codes to pay") {
    BarcodeScanner()
}
```

---

## Commerce & fintech

### `CartBadge` — live cart count

```ui
IconButton("cart") { navigate("/cart") }
CartBadge(cart.itemCount)   // patches one native Text layer on change
```

### `PriceTag` — formatted prices

```ui
PriceTag(price: product.price, sale: product.salePrice)
// Renders "$29.99" with "$49.99" struck through
```

### `QuantityStepper` — cart quantity

```ui
QuantityStepper(cart.qty, min: 1, max: 99) {
    cart.setQty(value)
}
```

---

## Custom UI from existing widgets

Flash also supports **`component`** — compose any built-in or Flash-exclusive
widget into reusable custom UI:

```ui
component FlightCard(airline: String, fare: Float) {
    Row {
        Text(airline).font(weight: .semibold)
        PriceTag(price: fare)
    }
    .padding(16)
    .background(.surface)
    .border_radius(12)
}

screen Home {
    List(flights) key: f.id { f in
        FlightCard(airline: f.airline, fare: f.fare)
            .on_tap { navigate("detail", id: f.id) }
    }
}
```

Use built-in + Flash-exclusive widgets as primitives. Use `component` for your
app's design system. The compiler inlines components at compile time — zero
runtime component overhead.

---

## Widget count

| Category | Count | Source |
|----------|-------|--------|
| Core + layout + display + input | ~55 | Flutter parity |
| Material + Cupertino | ~30 | Design systems |
| Platform (WebView, Map, Camera) | ~9 | Native FFI |
| **Flash-exclusive** | **38** | Compile-time reactivity |
| **Total** | **~120+** | Growing toward Flutter's 528 |

Flash-exclusive widgets are the competitive edge — they encode patterns that
take hours to build correctly in Flutter/RN, and compile to native perf.
