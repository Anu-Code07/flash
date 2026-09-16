//! Hot reload session — recompile `.ui`, diff IR, patch live UI, preserve state.

use std::sync::{Arc, Mutex};

use flash_ir::{
    hot_reload::{map_slot_values, plan_hot_reload, HotReloadPlan, RemountReason},
    HandlerId, ScreenIr,
};
use flash_platform::{HostCallbacks, InProcessHost, NativeRenderer};

use crate::live_renderer::{LiveRenderer, PatchOp};
use crate::native::{mount_screen, remount_props_native, remount_screen_native};
use crate::reactive::ReactiveEngine;
use crate::state::SlotStore;

/// Outcome of applying a hot reload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotReloadResult {
    pub kind: HotReloadKind,
    pub props_patched: usize,
    pub slots_preserved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotReloadKind {
    /// Tree unchanged — patched props only (<1s, state kept).
    HotReload,
    /// Structure changed — remounted tree, slot values preserved by name.
    HotRestart,
}

/// Dev session: reactive engine + live renderer + hot reload.
pub struct DevSession {
    pub engine: ReactiveEngine,
    pub renderer: LiveRenderer,
}

impl DevSession {
    pub fn mount(screen: ScreenIr) -> Self {
        let engine = ReactiveEngine::new(screen);
        let mut renderer = LiveRenderer::new();
        engine.mount_live(&mut renderer);
        Self { engine, renderer }
    }

    /// Recompile result — apply hot reload or hot restart.
    pub fn apply(&mut self, new_screen: ScreenIr) -> HotReloadResult {
        let plan = plan_hot_reload(&self.engine.screen, &new_screen);
        match plan {
            HotReloadPlan::PropsOnly => self.hot_reload_props(new_screen),
            HotReloadPlan::Remount { reason } => self.hot_restart(new_screen, reason),
        }
    }

    pub fn fire_handler(&mut self, handler_id: HandlerId) {
        self.engine.fire_handler(handler_id);
        self.engine.flush_live(&mut self.renderer);
    }

    fn hot_reload_props(&mut self, new_screen: ScreenIr) -> HotReloadResult {
        self.engine.screen = new_screen;
        self.renderer.patch_log.clear();
        self.engine.remount_props_live(&mut self.renderer);
        let props_patched = self
            .renderer
            .patch_log
            .iter()
            .filter(|p| matches!(p, PatchOp::SetProp { .. }))
            .count();
        HotReloadResult {
            kind: HotReloadKind::HotReload,
            props_patched,
            slots_preserved: true,
        }
    }

    fn hot_restart(&mut self, new_screen: ScreenIr, _reason: RemountReason) -> HotReloadResult {
        let old_slots = self.engine.screen.slots.clone();
        let old_values = self.engine.slots.ints.clone();
        let preserved = map_slot_values(&old_slots, &old_values, &new_screen.slots);

        self.engine.screen = new_screen;
        self.engine.slots = SlotStore::from_init(preserved);
        self.renderer.clear();
        self.engine.mount_live(&mut self.renderer);

        HotReloadResult {
            kind: HotReloadKind::HotRestart,
            props_patched: self.renderer.node_count(),
            slots_preserved: true,
        }
    }

    pub fn slot_values(&self) -> &[i64] {
        &self.engine.slots.ints
    }
}

struct SharedHost(Arc<Mutex<InProcessHost>>);

impl HostCallbacks for SharedHost {
    fn apply_ops(&mut self, ops: &[u8]) {
        self.0.lock().unwrap().apply_ops(ops);
    }
}

/// Native dev session — hot reload patches flow through `flash_host_apply_ops`.
pub struct NativeDevSession {
    pub engine: ReactiveEngine,
    pub renderer: NativeRenderer,
    host: Arc<Mutex<InProcessHost>>,
}

impl NativeDevSession {
    pub fn mount(screen: ScreenIr) -> Self {
        let host = Arc::new(Mutex::new(InProcessHost::default()));
        let engine = ReactiveEngine::new(screen);
        let mut renderer = NativeRenderer::new(Box::new(SharedHost(host.clone())));
        mount_screen(&engine, &mut renderer);
        Self {
            engine,
            renderer,
            host,
        }
    }

    pub fn apply(&mut self, new_screen: ScreenIr) -> HotReloadResult {
        let plan = plan_hot_reload(&self.engine.screen, &new_screen);
        match plan {
            HotReloadPlan::PropsOnly => self.hot_reload_props(new_screen),
            HotReloadPlan::Remount { reason } => self.hot_restart(new_screen, reason),
        }
    }

    pub fn fire_handler(&mut self, handler_id: HandlerId) {
        self.engine.fire_handler(handler_id);
        crate::native::flush_screen(&mut self.engine, &mut self.renderer);
    }

    pub fn slot_values(&self) -> &[i64] {
        &self.engine.slots.ints
    }

    pub fn describe_views(&self) -> Vec<String> {
        let host = self.host.lock().unwrap();
        host.views
            .iter()
            .map(|(handle, view)| {
                let text = view
                    .props
                    .get(&0)
                    .map(|v| format!("{:?}", v))
                    .unwrap_or_default();
                let frame = view
                    .frame
                    .map(|(x, y, w, h)| format!(" @({:.0},{:.0} {:.0}x{:.0})", x, y, w, h))
                    .unwrap_or_default();
                format!("view{} kind={} text={}{}", handle, view.kind, text, frame)
            })
            .collect()
    }

    pub fn view_count(&self) -> usize {
        self.host.lock().unwrap().views.len()
    }

    fn hot_reload_props(&mut self, new_screen: ScreenIr) -> HotReloadResult {
        self.engine.screen = new_screen;
        self.host.lock().unwrap().ops_log.clear();
        remount_props_native(&self.engine, &mut self.renderer);
        let props_patched = self
            .host
            .lock()
            .unwrap()
            .ops_log
            .iter()
            .filter(|op| matches!(op, flash_platform::DecodedOp::SetProp { .. }))
            .count();
        HotReloadResult {
            kind: HotReloadKind::HotReload,
            props_patched,
            slots_preserved: true,
        }
    }

    fn hot_restart(&mut self, new_screen: ScreenIr, _reason: RemountReason) -> HotReloadResult {
        let old_slots = self.engine.screen.slots.clone();
        let old_values = self.engine.slots.ints.clone();
        let preserved = map_slot_values(&old_slots, &old_values, &new_screen.slots);

        self.engine.screen = new_screen;
        self.engine.slots = SlotStore::from_init(preserved);
        self.host.lock().unwrap().ops_log.clear();
        self.host.lock().unwrap().views.clear();
        self.host.lock().unwrap().root_children.clear();
        self.renderer = NativeRenderer::new(Box::new(SharedHost(self.host.clone())));
        remount_screen_native(&self.engine, &mut self.renderer);

        HotReloadResult {
            kind: HotReloadKind::HotRestart,
            props_patched: self.host.lock().unwrap().views.len(),
            slots_preserved: true,
        }
    }
}
