use flash_driver::compile;
use flash_ir::HandlerId;
use flash_platform::{DecodedOp, HostCallbacks, InProcessHost, LayoutEngine};
use flash_runtime::{dispatch_handler, mount_screen, NativeDevSession, NativeSession, ReactiveEngine};
use std::sync::{Arc, Mutex};
use ui_suite::COUNTER_UI;

struct TraceHost(Arc<Mutex<InProcessHost>>);

impl HostCallbacks for TraceHost {
    fn apply_ops(&mut self, ops: &[u8]) {
        self.0.lock().unwrap().apply_ops(ops);
    }
}

#[test]
fn layout_engine_emits_frames_for_column() {
    let result = compile(COUNTER_UI).expect("compile");
    let frames = LayoutEngine::layout_screen(&result.ir.screens[0]);
    assert_eq!(frames.len(), 3);
    assert!(frames[0].width > 0.0);
    assert!(frames[1].height >= 22.0);
}

#[test]
fn native_mount_emits_set_frame_ops() {
    let result = compile(COUNTER_UI).expect("compile");
    let trace = Arc::new(Mutex::new(InProcessHost::default()));
    let engine = ReactiveEngine::new(result.ir.screens[0].clone());
    let mut renderer = flash_platform::NativeRenderer::new(Box::new(TraceHost(trace.clone())));
    mount_screen(&engine, &mut renderer);

    let frame_ops = trace
        .lock()
        .unwrap()
        .ops_log
        .iter()
        .filter(|op| matches!(op, DecodedOp::SetFrame { .. }))
        .count();
    assert_eq!(frame_ops, 3, "layout should emit SetFrame for each node");
}

#[test]
fn handler_round_trip_via_dispatch() {
    let result = compile(COUNTER_UI).expect("compile");
    let session = NativeSession::mount_shared(
        result.ir.screens[0].clone(),
        Box::new(InProcessHost::default()),
    );
    assert!(dispatch_handler(0));
    let s = session.lock().unwrap();
    assert_eq!(s.engine.slots.get_int(flash_ir::SlotId(0)), 1);
    NativeSession::unmount_shared(&session);
}

#[test]
fn native_dev_session_hot_reload_preserves_slots() {
    let result = compile(COUNTER_UI).expect("compile");
    let mut session = NativeDevSession::mount(result.ir.screens[0].clone());
    session.fire_handler(HandlerId(0));
    assert_eq!(session.slot_values()[0], 1);

    let mut edited = result.ir.screens[0].clone();
    edited.static_props.push(flash_ir::StaticProp {
        node: flash_ir::NodeId(2),
        key: flash_ir::PropKey::Title,
        value: flash_ir::IrExpr::Str("Plus".into()),
    });
    let reload = session.apply(edited);
    assert!(reload.slots_preserved);
    assert_eq!(session.slot_values()[0], 1);
}
