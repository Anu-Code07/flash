//! State slot storage with dirty tracking.

use flash_ir::SlotId;

#[derive(Clone, Debug)]
pub struct SlotStore {
    pub ints: Vec<i64>,
    pub strings: Vec<String>,
    pub bools: Vec<bool>,
    dirty: Vec<bool>,
}

impl SlotStore {
    pub fn new(num_slots: usize) -> Self {
        Self {
            ints: vec![0; num_slots],
            strings: vec![String::new(); num_slots],
            bools: vec![false; num_slots],
            dirty: vec![false; num_slots],
        }
    }

    pub fn from_init(ints: Vec<i64>) -> Self {
        let n = ints.len();
        Self {
            ints,
            strings: vec![String::new(); n],
            bools: vec![false; n],
            dirty: vec![false; n],
        }
    }

    pub fn get_int(&self, slot: SlotId) -> i64 {
        self.ints[slot.0 as usize]
    }

    pub fn set_int(&mut self, slot: SlotId, value: i64) {
        if self.ints[slot.0 as usize] != value {
            self.ints[slot.0 as usize] = value;
            self.dirty[slot.0 as usize] = true;
        }
    }

    pub fn increment(&mut self, slot: SlotId) {
        self.ints[slot.0 as usize] += 1;
        self.dirty[slot.0 as usize] = true;
    }

    pub fn drain_dirty(&mut self) -> Vec<SlotId> {
        let dirty_slots: Vec<SlotId> = self.dirty.iter().enumerate()
            .filter(|(_, &d)| d)
            .map(|(i, _)| SlotId(i as u32))
            .collect();
        for slot in &dirty_slots {
            self.dirty[slot.0 as usize] = false;
        }
        dirty_slots
    }
}
