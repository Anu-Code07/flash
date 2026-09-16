//! Hot reload — diff two `ScreenIr` snapshots and classify the update.

use crate::{NodeIr, PropKey, ScreenIr, SlotDef, StaticProp};

/// Result of comparing old vs new screen IR after a `.ui` recompile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotReloadPlan {
    /// Tree shape unchanged — patch props only, preserve slot values.
    PropsOnly,
    /// Node tree or handlers changed — remount UI, preserve slots by name.
    Remount { reason: RemountReason },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemountReason {
    NodeCountChanged,
    NodeKindChanged,
    ChildrenChanged,
    SlotCountChanged,
    HandlerCountChanged,
}

/// Compare screens and decide hot reload vs hot restart.
pub fn plan_hot_reload(old: &ScreenIr, new: &ScreenIr) -> HotReloadPlan {
    if old.slots.len() != new.slots.len() {
        return HotReloadPlan::Remount {
            reason: RemountReason::SlotCountChanged,
        };
    }
    if old.handlers.len() != new.handlers.len() {
        return HotReloadPlan::Remount {
            reason: RemountReason::HandlerCountChanged,
        };
    }
    if !same_node_tree(&old.nodes, &new.nodes) {
        return HotReloadPlan::Remount {
            reason: remount_reason_for_nodes(&old.nodes, &new.nodes),
        };
    }
    HotReloadPlan::PropsOnly
}

fn remount_reason_for_nodes(old: &[NodeIr], new: &[NodeIr]) -> RemountReason {
    if old.len() != new.len() {
        return RemountReason::NodeCountChanged;
    }
    for (a, b) in old.iter().zip(new.iter()) {
        if a.kind != b.kind {
            return RemountReason::NodeKindChanged;
        }
        if a.parent != b.parent || a.children != b.children {
            return RemountReason::ChildrenChanged;
        }
    }
    RemountReason::NodeCountChanged
}

fn same_node_tree(old: &[NodeIr], new: &[NodeIr]) -> bool {
    if old.len() != new.len() {
        return false;
    }
    old.iter().zip(new.iter()).all(|(a, b)| {
        a.kind == b.kind && a.parent == b.parent && a.children == b.children
    })
}

/// Static props whose values differ between compilations (by node + key).
pub fn diff_static_props(old: &[StaticProp], new: &[StaticProp]) -> Vec<(u32, PropKey)> {
    let mut changed = Vec::new();
    for np in new {
        let old_val = old
            .iter()
            .find(|p| p.node == np.node && p.key == np.key)
            .map(|p| format!("{:?}", p.value));
        let new_val = format!("{:?}", np.value);
        if old_val.as_deref() != Some(&new_val) {
            changed.push((np.node.0, np.key));
        }
    }
    for op in new {
        if !old.iter().any(|p| p.node == op.node && p.key == op.key) {
            changed.push((op.node.0, op.key));
        }
    }
    changed
}

/// Copy slot values from `old_store` into positions matching slot names in `new_slots`.
pub fn map_slot_values(
    old_slots: &[SlotDef],
    old_values: &[i64],
    new_slots: &[SlotDef],
) -> Vec<i64> {
    new_slots
        .iter()
        .map(|ns| {
            old_slots
                .iter()
                .position(|os| os.name == ns.name)
                .map(|i| old_values[i])
                .unwrap_or_else(|| match &ns.init {
                    crate::IrExpr::Int(v) => *v,
                    _ => 0,
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrExpr, NodeId, NodeIr, NodeKind, SlotDef, SlotId, StaticProp, UpdateOp};
    use flash_span::Symbol;
    use flash_stl::types::TypeKind;

    fn minimal_screen(text_label: &str) -> ScreenIr {
        ScreenIr {
            name: Symbol(1),
            slots: vec![SlotDef {
                name: Symbol(2),
                ty: TypeKind::Int,
                init: IrExpr::Int(0),
            }],
            nodes: vec![
                NodeIr {
                    kind: NodeKind::COLUMN,
                    parent: None,
                    children: vec![NodeId(1), NodeId(2)],
                    handler: None,
                },
                NodeIr {
                    kind: NodeKind::TEXT,
                    parent: Some(NodeId(0)),
                    children: vec![],
                    handler: None,
                },
                NodeIr {
                    kind: NodeKind::BUTTON,
                    parent: Some(NodeId(0)),
                    children: vec![],
                    handler: None,
                },
            ],
            static_props: vec![StaticProp {
                node: NodeId(2),
                key: PropKey::Title,
                value: IrExpr::Str(text_label.to_string()),
            }],
            update_ops: vec![UpdateOp {
                node: NodeId(1),
                key: PropKey::Text,
                expr: crate::ExprId(0),
                reads: vec![SlotId(0)],
            }],
            deps: crate::DepTable::build(1, &[UpdateOp {
                node: NodeId(1),
                key: PropKey::Text,
                expr: crate::ExprId(0),
                reads: vec![SlotId(0)],
            }]),
            handlers: vec![],
            actions: vec![],
            listens: vec![],
            exprs: vec![IrExpr::Slot(SlotId(0))],
        }
    }

    #[test]
    fn props_only_when_label_changes() {
        let old = minimal_screen("Increment");
        let new = minimal_screen("Add");
        assert_eq!(plan_hot_reload(&old, &new), HotReloadPlan::PropsOnly);
        assert!(!diff_static_props(&old.static_props, &new.static_props).is_empty());
    }

    #[test]
    fn remount_when_node_count_changes() {
        let old = minimal_screen("Go");
        let mut new = minimal_screen("Go");
        new.nodes.push(NodeIr {
            kind: NodeKind::TEXT,
            parent: Some(NodeId(0)),
            children: vec![],
            handler: None,
        });
        assert!(matches!(
            plan_hot_reload(&old, &new),
            HotReloadPlan::Remount { reason: RemountReason::NodeCountChanged }
        ));
    }

    #[test]
    fn preserve_slots_by_name() {
        let old_slots = vec![
            SlotDef {
                name: Symbol(10),
                ty: TypeKind::Int,
                init: IrExpr::Int(0),
            },
            SlotDef {
                name: Symbol(11),
                ty: TypeKind::Int,
                init: IrExpr::Int(0),
            },
        ];
        let new_slots = vec![
            SlotDef {
                name: Symbol(11),
                ty: TypeKind::Int,
                init: IrExpr::Int(0),
            },
            SlotDef {
                name: Symbol(12),
                ty: TypeKind::Int,
                init: IrExpr::Int(99),
            },
        ];
        let mapped = map_slot_values(&old_slots, &[3, 7], &new_slots);
        assert_eq!(mapped, vec![7, 99]);
    }
}
