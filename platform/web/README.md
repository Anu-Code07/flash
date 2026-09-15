# Flash Web Backend

Web support via WASM + DOM shim (Phase 9).

## Integration

```
Rust WASM (wasm32-unknown-unknown)
    ↓ wasm-bindgen
JS shim (~few KB)
    ↓
DOM API
```

## Frame driver

`requestAnimationFrame` — same command buffer, one crossing per frame.

## Note

Web is secondary to mobile (iOS/Android). The same UI IR and reactive
dependency graph apply — no virtual DOM, fine-grained DOM updates only.
