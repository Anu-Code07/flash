use flash_driver::compile;
use flash_runtime::{DevSession, HotReloadKind};
use ui_suite::COUNTER_UI;

#[test]
fn hot_reload_preserves_slot_on_label_change() {
    let result = compile(COUNTER_UI).expect("compile");
    let screen = result.ir.screens[0].clone();
    let mut session = DevSession::mount(screen);

    session.fire_handler(flash_ir::HandlerId(0));
    assert_eq!(session.slot_values()[0], 1);

    let mut edited = COUNTER_UI.replace("Increment", "Add");
    edited = edited.replace("Count:", "Total:");
    let new_result = compile(&edited).expect("recompile");
    let new_screen = new_result.ir.screens[0].clone();

    let reload = session.apply(new_screen);
    assert_eq!(reload.kind, HotReloadKind::HotReload);
    assert!(reload.slots_preserved);
    assert_eq!(session.slot_values()[0], 1);

    let text = session
        .renderer
        .prop(flash_ir::NodeId(1), flash_ir::PropKey::Text)
        .and_then(|v| match v {
            flash_runtime::PropValue::Str(s) => Some(s.clone()),
            _ => None,
        });
    assert_eq!(text.as_deref(), Some("Total: 1"));
}

#[test]
fn hot_restart_on_structure_change() {
    let result = compile(COUNTER_UI).expect("compile");
    let screen = result.ir.screens[0].clone();
    let mut session = DevSession::mount(screen);
    session.fire_handler(flash_ir::HandlerId(0));

    let with_row = COUNTER_UI.replace("Column {", "Row {");
    let new_result = compile(&with_row).expect("recompile");
    let new_screen = new_result.ir.screens[0].clone();

    let reload = session.apply(new_screen);
    assert_eq!(reload.kind, HotReloadKind::HotRestart);
    assert_eq!(session.slot_values()[0], 1);
}
