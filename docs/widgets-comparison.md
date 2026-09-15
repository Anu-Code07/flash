# Flash vs Flutter vs React Native — Widget Comparison

**Last updated:** 2026-09-15

A practical inventory of what each framework ships for mobile UI, where Flash
currently stands, and what to add next.

---

## Scale of each ecosystem

| Framework | Widget count | Philosophy |
|-----------|-------------|------------|
| **Flutter** | ~528 public widgets | Full design system (Material 3 + Cupertino) + base layout/animation/scrolling |
| **React Native** | ~25 core components | Thin native wrappers; community fills the rest (React Navigation, Paper, etc.) |
| **Flash** | 10 built-ins (Phase 2) | Native widget mapping; modifiers replace most styling widgets |

Flutter bundles everything. React Native ships a minimal core and expects npm
packages for navigation, forms, and design systems. Flash targets the RN
philosophy (native views) with a richer built-in STL and modifier system.

---

## React Native — Core Components

Source: [reactnative.dev/docs/components-and-apis](https://reactnative.dev/docs/components-and-apis)

### Basic (every app uses these)

| RN Component | Native iOS | Native Android | Flash equivalent |
|-------------|-----------|----------------|------------------|
| `View` | `UIView` | `ViewGroup` | `Column` / `Row` / `Stack` |
| `Text` | `UILabel` | `TextView` | `Text` |
| `Image` | `UIImageView` | `ImageView` | `Image` |
| `TextInput` | `UITextField` | `EditText` | `TextField` |
| `Pressable` | touch handler | touch handler | `Button` + `.on_tap` |
| `ScrollView` | `UIScrollView` | `ScrollView` | `ScrollView` |
| `StyleSheet` | — | — | modifier chains (`.padding`, `.color`, …) |

### UI controls

| RN Component | Flash equivalent | Status |
|-------------|-----------------|--------|
| `Button` | `Button` | ✅ |
| `Switch` | `Switch` | ❌ Phase 5c |
| `ActivityIndicator` | `Loading` | ✅ |
| `Modal` | `Modal` / `Sheet` | ❌ Phase 5c |
| `Alert` | platform dialog via Rust | ❌ Phase 5 |
| `KeyboardAvoidingView` | `.keyboard_avoiding()` modifier | 🟡 modifier defined |
| `StatusBar` | `status_bar_*()` STL | 🟡 STL defined |
| `RefreshControl` | `.pull_to_refresh()` | ❌ Phase 5c |

### Lists

| RN Component | Native mapping | Flash equivalent |
|-------------|---------------|------------------|
| `FlatList` | `UICollectionView` / `RecyclerView` | `List` (keyed) |
| `SectionList` | sectioned collection | `SectionList` | ❌ Phase 5c |

### Platform-specific

| RN Component | Platform | Flash approach |
|-------------|----------|----------------|
| `ActionSheetIOS` | iOS | `action_sheet()` STL → UIAlertController |
| `DrawerLayoutAndroid` | Android | `Drawer` component |
| `BackHandler` | Android | `navigate_back()` + hardware back STL |
| `ToastAndroid` | Android | `toast()` STL |
| `PermissionsAndroid` | Android | Rust + platform FFI |

### RN ecosystem (not core, but expected in production apps)

| Library | Purpose | Flash equivalent |
|---------|---------|------------------|
| React Navigation | stack/tab/drawer nav | `navigate()` STL + route table |
| react-native-reanimated | 60fps animations | `.with_animation()` → Core Animation / Animator |
| react-native-gesture-handler | gestures | `.on_swipe`, `.on_long_press` modifiers |
| react-native-safe-area-context | safe areas | `.safe_area()` modifier + `safe_area_*()` STL |
| @gorhom/bottom-sheet | bottom sheets | `Sheet` component |
| react-native-paper / native-base | design system | design tokens + modifiers |

**Takeaway:** React Native's core is ~12 components. Flash already covers 8/12.
The gap is `Switch`, `Modal`, `SectionList`, and pull-to-refresh.

---

## Flutter — Widget Catalog

Source: [docs.flutter.dev/reference/widgets](https://docs.flutter.dev/reference/widgets),
[widgets.json](https://github.com/flutter/tools_metadata/blob/master/resources/catalog/widgets.json)

### By library

| Library | Count | Role |
|---------|-------|------|
| Base (`widgets`) | ~296 | Layout, painting, scrolling, animation, gestures |
| `material` | ~182 | Material 3 design system |
| `cupertino` | ~50 | iOS HIG design system |

### Flutter categories → Flash mapping

#### Layout (Flutter base)

| Flutter | Flash | Notes |
|---------|-------|-------|
| `Column` | `Column` | ✅ |
| `Row` | `Row` | ✅ |
| `Stack` | `Stack` | ✅ |
| `Expanded` / `Flexible` | `.flex()` modifier | ✅ modifier |
| `Padding` | `.padding()` modifier | ✅ modifier |
| `Center` | `.align(.center)` modifier | ✅ modifier |
| `SizedBox` | `.width()` / `.height()` | ✅ modifier |
| `Wrap` | `Wrap` | ❌ Phase 5b (flex layout) |
| `GridView` | `Grid` | ❌ Phase 6 |
| `CustomScrollView` | `ScrollView` + slivers | ❌ Phase 6 |
| `LayoutBuilder` | compile-time layout | Flash avoids runtime layout builders |

#### Display

| Flutter | Flash | Notes |
|---------|-------|-------|
| `Text` | `Text` | ✅ |
| `RichText` | `Text` with spans | ❌ Phase 5c |
| `Image` / `Image.network` | `Image` | ✅ |
| `Icon` | `Icon` | ❌ Phase 5c |
| `CircleAvatar` | `Avatar` | ❌ Phase 5c |
| `Divider` | `Divider` | ❌ Phase 5c |
| `Card` | modifier extension `.card()` | ✅ extension pattern |
| `Badge` | `Badge` | ❌ Phase 6 |

#### Input

| Flutter | Flash | Notes |
|---------|-------|-------|
| `ElevatedButton` / `TextButton` | `Button` | ✅ (style via modifiers) |
| `TextField` / `TextFormField` | `TextField` | ✅ |
| `Checkbox` | `Checkbox` | ❌ Phase 5c |
| `Radio` | `Radio` | ❌ Phase 5c |
| `Switch` | `Switch` | ❌ Phase 5c |
| `Slider` | `Slider` | ❌ Phase 6 |
| `DropdownButton` | `Picker` | ❌ Phase 6 |
| `DatePicker` | platform picker via FFI | ❌ Phase 6 |

#### Lists & scrolling

| Flutter | Flash | Notes |
|---------|-------|-------|
| `ListView` | `List` | ✅ (keyed virtualization) |
| `ListView.builder` | `List(data) key: …` | ✅ |
| `GridView` | `Grid` | ❌ Phase 6 |
| `SingleChildScrollView` | `ScrollView` | ✅ |
| `RefreshIndicator` | `.pull_to_refresh()` | ❌ Phase 5c |

#### Navigation (Material + Cupertino)

| Flutter | Flash | Notes |
|---------|-------|-------|
| `Navigator` / `MaterialPageRoute` | `navigate()` STL | 🟡 STL defined |
| `BottomNavigationBar` | `TabBar` / tab example | 🟡 example exists |
| `CupertinoTabBar` | `TabBar` (iOS style) | ❌ Phase 5c |
| `AppBar` / `CupertinoNavigationBar` | `AppBar` | ❌ Phase 5c |
| `Drawer` | `Drawer` | ❌ Phase 6 |
| `BottomSheet` / `CupertinoActionSheet` | `Sheet` / `ActionSheet` | ❌ Phase 5c |

#### Feedback & overlay

| Flutter | Flash | Notes |
|---------|-------|-------|
| `AlertDialog` / `CupertinoAlertDialog` | `Alert` via Rust | ❌ Phase 5 |
| `SnackBar` | `toast()` STL | ❌ Phase 5c |
| `CircularProgressIndicator` | `Loading` | ✅ |
| `LinearProgressIndicator` | `ProgressBar` | ❌ Phase 5c |
| `Tooltip` | `Tooltip` | ❌ Phase 6 |

#### Animation (Flutter's strength — Flash's planned strength)

| Flutter | Flash | Notes |
|---------|-------|-------|
| `AnimatedContainer` | `.with_animation()` modifier | 🟡 IR types exist |
| `AnimatedOpacity` | `.animate_opacity()` | 🟡 modifier defined |
| `AnimatedPositioned` | `.animate_offset()` | 🟡 modifier defined |
| `Hero` | shared-element transition | ❌ Phase 7 |
| `PageRouteBuilder` | platform page transition | ❌ Phase 5c |
| `AnimationController` | platform animator handle | 🟡 `PlatformAnimator` trait |

**Takeaway:** Flutter has 528 widgets but most are styling variants of ~40
primitives. Flash's modifier system collapses `Padding`, `Center`, `SizedBox`,
`ColoredBox`, `DecoratedBox`, etc. into chains — so 10 components + 30 modifiers
already cover what RN ships as 12 components + StyleSheet.

---

## Coverage scorecard (mobile MVP)

What a typical production mobile app needs vs what each framework provides
out of the box:

| Capability | React Native | Flutter | Flash (now) | Flash (Phase 5c target) |
|-----------|-------------|---------|-------------|------------------------|
| Layout (flex) | ✅ View + flex | ✅ Column/Row/Stack | ✅ Column/Row/Stack | ✅ + Wrap |
| Text | ✅ | ✅ | ✅ | ✅ + RichText |
| Images | ✅ | ✅ | ✅ | ✅ |
| Buttons | ✅ | ✅ | ✅ | ✅ |
| Text input | ✅ | ✅ | ✅ | ✅ |
| Lists (virtualized) | ✅ FlatList | ✅ ListView.builder | 🟡 parser only | ✅ |
| Tabs / bottom nav | 📦 package | ✅ | 🟡 example | ✅ TabBar |
| Stack navigation | 📦 package | ✅ Navigator | 🟡 STL only | ✅ |
| Modal / sheet | ✅ Modal | ✅ | ❌ | ✅ |
| Pull to refresh | ✅ | ✅ | ❌ | ✅ |
| Switch / checkbox | ✅ | ✅ | ❌ | ✅ |
| Loading spinner | ✅ | ✅ | ✅ | ✅ |
| Safe area | 📦 package | ✅ SafeArea | 🟡 modifier | ✅ |
| Haptics | 📦 package | 📦 HapticFeedback | 🟡 STL | ✅ |
| Animations (60fps) | 📦 reanimated | ✅ | 🟡 IR only | ✅ platform animators |
| Alerts / toasts | ✅ Alert | ✅ SnackBar | ❌ | ✅ via STL |
| Icons | 📦 package | ✅ Icons | ❌ | ✅ Icon component |
| App bar | 📦 package | ✅ AppBar | ❌ | ✅ AppBar |
| Platform look (iOS/Android) | manual styling | ✅ Material + Cupertino | native widgets自动 | native widgets |

---

## Flash's architectural advantage

### What Flutter does with widgets, Flash does with modifiers

Flutter needs separate widget classes for styling:

```dart
// Flutter — 4 widget layers for one Text
Padding(
  padding: EdgeInsets.all(16),
  child: Container(
    color: Colors.blue,
    child: Text('Hello', style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
  ),
)
```

Flash collapses this into one node + modifier chain:

```ui
Text("Hello")
    .font(size: 24, weight: .bold)
    .color(.primary)
    .padding(16)
    .background(.surface)
```

This is why Flash ships 10 components vs Flutter's 528 — the modifier system
replaces ~200 Flutter styling/layout widgets.

### What React Native does with packages, Flash does with STL

| RN pattern | Flash STL |
|-----------|-----------|
| `react-native-safe-area-context` | `.safe_area()`, `safe_area_top()` |
| `react-native-haptic-feedback` | `haptic_light()`, `haptic_medium()` |
| `@react-navigation/native` | `navigate()`, `navigate_back()` |
| `react-native-reanimated` | `.with_animation()`, `.animate_opacity()` |
| `Share` from react-native | `share()` STL |

---

## Recommended Phase 5c component additions

Priority order based on Flutter/RN mobile app frequency:

### P0 — every app needs these

| Component | Flutter analog | RN analog | Native mapping |
|-----------|---------------|-----------|----------------|
| `Switch` | `Switch` | `Switch` | UISwitch / SwitchMaterial |
| `AppBar` | `AppBar` / `CupertinoNavigationBar` | header in navigation lib | UINavigationBar / Toolbar |
| `TabBar` | `BottomNavigationBar` / `CupertinoTabBar` | tab navigator | UITabBar / BottomNavigationView |
| `Modal` | `showModalBottomSheet` | `Modal` | UIViewController modal / DialogFragment |
| `Icon` | `Icon` | icon font / vector | SF Symbols / Material Icons |

### P1 — most apps need these

| Component | Flutter analog | RN analog |
|-----------|---------------|-----------|
| `Checkbox` | `Checkbox` | checkbox lib |
| `Divider` | `Divider` | `View` with border |
| `ProgressBar` | `LinearProgressIndicator` | `ProgressBarAndroid` |
| `SectionList` | `ListView` with headers | `SectionList` |
| `Sheet` | `BottomSheet` | `@gorhom/bottom-sheet` |
| `Avatar` | `CircleAvatar` | image + border radius |
| `Badge` | `Badge` | badge lib |

### P2 — nice to have

| Component | Flutter analog | RN analog |
|-----------|---------------|-----------|
| `Slider` | `Slider` | `@react-native-community/slider` |
| `Picker` | `DropdownButton` | picker lib |
| `Grid` | `GridView` | `FlatList numColumns` |
| `WebView` | `WebView` | `react-native-webview` |
| `Map` | `GoogleMap` | `react-native-maps` |

Platform-specific widgets (`Map`, `Camera`, `WebView`) should stay as Rust +
FFI extensions, not built-in `.ui` components.

---

## Summary

| | React Native | Flutter | Flash |
|--|-------------|---------|-------|
| **Core components** | ~12 | ~40 primitives + 488 styling variants | 10 + 30 modifiers |
| **Design system** | Community (Paper, NativeBase) | Built-in (Material + Cupertino) | Design tokens + native widgets |
| **Navigation** | React Navigation (npm) | Built-in Navigator | `navigate()` STL + route table |
| **Animation** | Reanimated (npm) | Built-in AnimationController | Platform animators (zero FFI/frame) |
| **Lists** | FlatList, SectionList | ListView, GridView | List (keyed), SectionList planned |
| **Strength** | Familiar React model, huge ecosystem | Complete widget catalog, consistent look | Native perf, compile-time reactivity, minimal runtime |
| **Weakness** | Bridge (old arch), package sprawl | Custom renderer, large runtime | Pre-1.0, smaller component set |

Flash does not need 528 widgets. It needs ~25 well-mapped native primitives,
a rich modifier STL, and platform animators — which covers the same mobile
surface area as React Native core + the most-used community packages, with
better performance than both.
