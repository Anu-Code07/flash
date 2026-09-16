use flash_driver::compile;
use flash_ir::HandlerId;
use flash_platform::{
    CommandBuffer, CommandOp, DecodedValue, HostCallbacks, InProcessHost, NodeKind, PropKey,
    PropValue,
};
use flash_runtime::NativeSession;
use ui_suite::COUNTER_UI;

#[test]
fn native_session_mounts_counter() {
    let result = compile(COUNTER_UI).expect("compile");
    let _session = NativeSession::mount(result.ir.screens[0].clone(), Box::new(InProcessHost::default()));
}

#[test]
fn native_session_preserves_state_on_handler() {
    let result = compile(COUNTER_UI).expect("compile");
    let mut session =
        NativeSession::mount(result.ir.screens[0].clone(), Box::new(InProcessHost::default()));
    session.fire_handler(HandlerId(0));
    assert_eq!(session.engine.slots.get_int(flash_ir::SlotId(0)), 1);
}

#[test]
fn in_process_host_builds_view_hierarchy() {
    let mut host = InProcessHost::default();
    let mut buf = CommandBuffer::new();
    buf.ops.push(CommandOp::Create {
        kind: NodeKind::COLUMN,
        handle: 1,
    });
    buf.ops.push(CommandOp::Create {
        kind: NodeKind::TEXT,
        handle: 2,
    });
    buf.ops.push(CommandOp::SetProp {
        handle: 2,
        key: PropKey::Text,
        value: PropValue::Str("Hello native".into()),
    });
    buf.ops.push(CommandOp::InsertChild {
        parent: 1,
        child: 2,
        index: 0,
    });
    host.apply_ops(&buf.encode());

    assert_eq!(host.views.len(), 2);
    assert!(matches!(
        host.views.get(&2).unwrap().props.get(&0),
        Some(DecodedValue::Str(s)) if s == "Hello native"
    ));
}
