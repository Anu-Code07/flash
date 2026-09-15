//! Flash-exclusive widgets — solve real mobile problems Flutter/RN make painful.
//!
//! These leverage compile-time `@provider` reactivity, fine-grained native updates,
//! and platform FFI. They are not thin wrappers; each collapses a common pattern
//! that normally takes 50–200 lines of boilerplate.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    // ── Async & reactive state (Flash's core advantage) ───────────────────
    widget!(92, "AsyncView", WidgetCategory::Flash, "Renders Loading/Empty/Error/Data slots for an @provider async field — replaces manual match blocks", "FlashAsyncView", "FlashAsyncView", true, false, PrimaryProp::None, false),
    widget!(93, "ProviderScope", WidgetCategory::Flash, "Injects an @provider into a subtree (Riverpod ProviderScope equivalent)", "FlashProviderScope", "FlashProviderScope", true, false, PrimaryProp::None, false),
    widget!(94, "BoundText", WidgetCategory::Flash, "Auto-watches a provider field — no ${} interpolation needed", "UILabel", "TextView", false, false, PrimaryProp::Text, true),
    widget!(95, "ListenEffect", WidgetCategory::Flash, "Runs a side-effect handler when a provider field changes (widget form of @listen)", "FlashListenEffect", "FlashListenEffect", false, true, PrimaryProp::None, false),

    // ── Screen patterns every production app needs ──────────────────────────
    widget!(96, "EmptyState", WidgetCategory::Flash, "Icon + title + message + optional CTA for empty lists/search results", "FlashEmptyState", "FlashEmptyState", true, true, PrimaryProp::Title, true),
    widget!(97, "ErrorState", WidgetCategory::Flash, "Error message + retry button wired to an @action — standard error recovery UI", "FlashErrorState", "FlashErrorState", true, true, PrimaryProp::Text, true),
    widget!(98, "BottomCTA", WidgetCategory::Flash, "Fixed bottom bar with primary action, floats above keyboard (checkout, continue)", "FlashBottomCTA", "FlashBottomCTA", true, true, PrimaryProp::Title, true),
    widget!(99, "OnboardingSlide", WidgetCategory::Flash, "Full-screen onboarding page: image, title, body, next/skip actions", "FlashOnboardingSlide", "FlashOnboardingSlide", true, true, PrimaryProp::Title, true),

    // ── Forms & validation (fintech, auth, signup flows) ───────────────────
    widget!(100, "PinField", WidgetCategory::Flash, "OTP / PIN code input with auto-advance, masking, and paste support", "FlashPinField", "FlashPinField", false, true, PrimaryProp::Value, false),
    widget!(101, "PhoneField", WidgetCategory::Flash, "International phone input with country picker and E.164 formatting", "FlashPhoneField", "FlashPhoneField", false, true, PrimaryProp::Value, false),
    widget!(102, "CurrencyField", WidgetCategory::Flash, "Locale-aware money input with symbol, decimals, and max amount", "FlashCurrencyField", "FlashCurrencyField", false, true, PrimaryProp::Value, false),
    widget!(103, "FormField", WidgetCategory::Flash, "Label + input + inline validation error bound to @provider field", "FlashFormField", "FlashFormField", true, false, PrimaryProp::Title, true),
    widget!(104, "ValidatedInput", WidgetCategory::Flash, "TextField that shows/hides error from provider validation state", "FlashValidatedInput", "FlashValidatedInput", false, true, PrimaryProp::Value, false),

    // ── Lists & loading UX ─────────────────────────────────────────────────
    widget!(105, "SkeletonList", WidgetCategory::Flash, "Shimmer placeholder rows while async data loads — no layout shift", "FlashSkeletonList", "FlashSkeletonList", false, false, PrimaryProp::None, false),
    widget!(106, "InfiniteScroll", WidgetCategory::Flash, "Paginated list that auto-calls @action loadMore at scroll end", "FlashInfiniteScroll", "FlashInfiniteScroll", true, false, PrimaryProp::None, false),
    widget!(107, "SwipeActionRow", WidgetCategory::Flash, "List row with swipe-to-delete/archive actions (mail, todo apps)", "FlashSwipeRow", "FlashSwipeRow", true, true, PrimaryProp::Title, true),
    widget!(108, "StickySectionHeader", WidgetCategory::Flash, "Section header that pins while scrolling (contacts, settings)", "FlashStickyHeader", "FlashStickyHeader", true, false, PrimaryProp::Title, true),
    widget!(109, "PullToRefresh", WidgetCategory::Flash, "Pull-to-refresh bound to an @provider @action — no manual wiring", "UIRefreshControl", "SwipeRefreshLayout", true, false, PrimaryProp::None, false),

    // ── Platform, connectivity & security ───────────────────────────────────
    widget!(110, "OfflineBanner", WidgetCategory::Flash, "Auto-shows banner when network drops, hides on reconnect", "FlashOfflineBanner", "FlashOfflineBanner", true, false, PrimaryProp::Text, false),
    widget!(111, "BiometricGate", WidgetCategory::Flash, "Face ID / fingerprint unlock overlay for banking & wallet apps", "FlashBiometricGate", "FlashBiometricGate", true, true, PrimaryProp::None, false),
    widget!(112, "SecureScreen", WidgetCategory::Flash, "Blocks screenshots and screen recording on sensitive screens", "FlashSecureScreen", "FlashSecureScreen", true, false, PrimaryProp::None, false),
    widget!(113, "PermissionGate", WidgetCategory::Flash, "Camera/location/notification permission prompt with rationale UI", "FlashPermissionGate", "FlashPermissionGate", true, true, PrimaryProp::Title, true),
    widget!(114, "AdaptiveContainer", WidgetCategory::Flash, "Renders iOS or Android child subtree based on platform — one screen, two looks", "FlashAdaptiveContainer", "FlashAdaptiveContainer", true, false, PrimaryProp::None, false),

    // ── Actions & navigation helpers ────────────────────────────────────────
    widget!(115, "HapticButton", WidgetCategory::Flash, "Button with built-in haptic feedback on tap — no manual haptic_light() calls", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(116, "DeepLinkButton", WidgetCategory::Flash, "Navigates via deep link URI with fallback route", "FlashDeepLinkButton", "FlashDeepLinkButton", false, true, PrimaryProp::Title, true),
    widget!(117, "ShareButton", WidgetCategory::Flash, "One-tap native share sheet for text, URL, or image", "UIActivityViewController", "Intent", false, true, PrimaryProp::Text, true),
    widget!(118, "RetryButton", WidgetCategory::Flash, "Retry button with loading spinner while @action re-runs", "FlashRetryButton", "FlashRetryButton", false, true, PrimaryProp::Title, false),

    // ── Media & content ─────────────────────────────────────────────────────
    widget!(119, "LazyImage", WidgetCategory::Flash, "Loads image only when scrolled into viewport — saves bandwidth & memory", "FlashLazyImage", "FlashLazyImage", false, false, PrimaryProp::Src, true),
    widget!(120, "CopyableText", WidgetCategory::Flash, "Tap-to-copy text with toast confirmation (referral codes, addresses)", "FlashCopyableText", "FlashCopyableText", false, true, PrimaryProp::Text, true),
    widget!(121, "CountdownLabel", WidgetCategory::Flash, "Live countdown for OTP expiry, flash sales, auction timers", "FlashCountdownLabel", "FlashCountdownLabel", false, false, PrimaryProp::Value, false),

    // ── Commerce & fintech ──────────────────────────────────────────────────
    widget!(122, "CartBadge", WidgetCategory::Flash, "Live cart item count badge bound to @provider — updates one native layer", "FlashCartBadge", "FlashCartBadge", false, false, PrimaryProp::Value, false),
    widget!(123, "PriceTag", WidgetCategory::Flash, "Locale-formatted price with strike-through sale price support", "FlashPriceTag", "FlashPriceTag", false, false, PrimaryProp::Value, true),
    widget!(124, "QuantityStepper", WidgetCategory::Flash, "+/− stepper for cart/checkout quantity with min/max bounds", "FlashQuantityStepper", "FlashQuantityStepper", false, true, PrimaryProp::Value, false),

    // ── Keyboard & layout ───────────────────────────────────────────────────
    widget!(125, "KeyboardSafe", WidgetCategory::Flash, "Combines safe area + keyboard avoidance in one widget (chat, forms)", "FlashKeyboardSafe", "FlashKeyboardSafe", true, false, PrimaryProp::None, false),
    widget!(126, "BlurSheet", WidgetCategory::Flash, "Bottom sheet with native blur backdrop (iOS frosted glass feel)", "FlashBlurSheet", "FlashBlurSheet", true, false, PrimaryProp::None, false),

    // ── Auth & onboarding flows ─────────────────────────────────────────────
    widget!(127, "RouteGuard", WidgetCategory::Flash, "Redirects to login when auth @provider says logged out", "FlashRouteGuard", "FlashRouteGuard", true, false, PrimaryProp::None, false),
    widget!(128, "RateAppCard", WidgetCategory::Flash, "In-app review prompt card with dismiss and rate actions", "FlashRateAppCard", "FlashRateAppCard", true, true, PrimaryProp::Title, false),
    widget!(129, "SocialLoginRow", WidgetCategory::Flash, "Apple/Google/email sign-in button row for auth screens", "FlashSocialLoginRow", "FlashSocialLoginRow", true, true, PrimaryProp::None, false),
];
