use flash_driver::compile;
use flash_ir::{HandlerId, NodeId, PropKey, SlotId};
use flash_runtime::{MockRenderer, ReactiveEngine, RenderCall};

const PROVIDER_COUNTER: &str = r#"
@provider Counter {
    @state count: Int = 0
    @state high_score: Int = 0

    @action increment() {
        count++
        high_score = count
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
        Button("+") { counter.increment() }
        Button("Reset") { counter.reset() }
    }
}
"#;

#[test]
fn provider_counter_ir() {
    let result = compile(PROVIDER_COUNTER).expect("should compile");
    let screen = &result.ir.screens[0];

    assert_eq!(screen.slots.len(), 2);
    assert_eq!(screen.actions.len(), 2);
    assert_eq!(screen.actions[0].writes.len(), 2, "increment writes count + high_score");
    assert_eq!(screen.deps.ops_for(SlotId(0)).len(), 1);
    assert_eq!(screen.deps.ops_for(SlotId(1)).len(), 1);
    assert!(result.ir_text.contains("ACTIONS"));
    assert!(result.ir_text.contains("counter.count"));
}

#[test]
fn provider_increment_updates_both_slots() {
    let result = compile(PROVIDER_COUNTER).expect("should compile");
    let screen = result.ir.screens[0].clone();

    let mut engine = ReactiveEngine::new(screen);
    let mut renderer = MockRenderer::new();
    engine.mount(&mut renderer);
    renderer.clear_log();

    // Find increment button handler
    let inc_handler = engine.screen.handlers.iter()
        .find(|h| matches!(h.body, flash_ir::HandlerBody::InvokeAction(0)))
        .expect("increment handler");
    engine.fire_handler(inc_handler.id);
    engine.flush(&mut renderer);

    let set_props = renderer.log().iter()
        .filter(|c| matches!(c, RenderCall::SetProp { .. }))
        .count();
    assert_eq!(set_props, 2, "increment should update count and high_score texts");
}

#[test]
fn provider_reset_updates_one_slot() {
    let result = compile(PROVIDER_COUNTER).expect("should compile");
    let screen = result.ir.screens[0].clone();

    let mut engine = ReactiveEngine::new(screen);
    let mut renderer = MockRenderer::new();
    engine.mount(&mut renderer);

    // increment first
    engine.fire_handler(HandlerId(0));
    engine.flush(&mut renderer);
    renderer.clear_log();

    // reset
    let reset_handler = engine.screen.handlers.iter()
        .find(|h| matches!(h.body, flash_ir::HandlerBody::InvokeAction(1)))
        .expect("reset handler");
    engine.fire_handler(reset_handler.id);
    engine.flush(&mut renderer);

    let set_props = renderer.log().iter()
        .filter(|c| matches!(c, RenderCall::SetProp { node: NodeId(1), key: PropKey::Text, .. }))
        .count();
    assert_eq!(set_props, 1, "reset only updates count text");
}
