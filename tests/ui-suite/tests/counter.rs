use flash_driver::compile;
use flash_ir::{HandlerId, NodeId, PropKey, SlotId};
use flash_runtime::{MockRenderer, PropValue, ReactiveEngine, RenderCall};
use ui_suite::COUNTER_UI;

#[test]
fn canonical_counter_ir() {
    let result = compile(COUNTER_UI).expect("should compile");
    let screen = &result.ir.screens[0];

    assert_eq!(screen.slots.len(), 1);
    assert_eq!(result.ir_text.contains("count"), true);

    // Text depends on count
    assert_eq!(screen.deps.ops_for(SlotId(0)).len(), 1);
    assert_eq!(screen.update_ops[0].node, NodeId(1));
    assert_eq!(screen.update_ops[0].key, PropKey::Text);

    // Button title is static
    assert!(screen.static_props.iter().any(|p| p.node == NodeId(2) && p.key == PropKey::Title));

    // Handler writes count
    assert_eq!(screen.handlers[0].writes, vec![SlotId(0)]);
}

#[test]
fn single_mutation_touches_exactly_one_property() {
    let result = compile(COUNTER_UI).expect("should compile");
    let screen = result.ir.screens[0].clone();

    let mut engine = ReactiveEngine::new(screen);
    let mut renderer = MockRenderer::new();
    engine.mount(&mut renderer);
    renderer.clear_log();

    engine.fire_handler(HandlerId(0));
    engine.flush(&mut renderer);

    assert_eq!(renderer.log().len(), 2);
    assert!(matches!(
        &renderer.log()[0],
        RenderCall::SetProp { node: NodeId(1), key: PropKey::Text, value: PropValue::Str(s) }
            if s == "Count: 1"
    ));
    assert!(matches!(&renderer.log()[1], RenderCall::Commit));
}

#[test]
fn ir_output_contains_dependencies() {
    let result = compile(COUNTER_UI).expect("should compile");
    assert!(result.ir_text.contains("SCREEN Home"));
    assert!(result.ir_text.contains("count"));
    assert!(result.ir_text.contains("DEPENDENCIES"));
    assert!(result.ir_text.contains("slot0(count)"));
}
