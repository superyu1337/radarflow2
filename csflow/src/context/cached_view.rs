use ::std::sync::{atomic::{AtomicU8, Ordering, AtomicI32}, Arc};

use memflow::prelude::v1::*;

pub const INVALIDATE_ALWAYS: u8 = 0b00000001;
pub const INVALIDATE_TICK: u8 = 0b00000010;

pub struct ExternallyControlledValidator {
    validator_next_flags: Arc<AtomicU8>,
    validator_tick_count: Arc<AtomicI32>,
}

impl ExternallyControlledValidator {
    pub fn new() -> Self {
        Self {
            validator_next_flags: Arc::new(AtomicU8::new(INVALIDATE_ALWAYS)),
            validator_tick_count: Arc::new(AtomicI32::new(0)),
        }
    }

    pub fn set_next_flags(&mut self, flags: u8) {
        self.validator_next_flags
            .store(flags as u8, Ordering::SeqCst);
    }

    pub fn set_tick_count(&mut self, tick_count: i32) {
        self.validator_tick_count
            .store(tick_count, Ordering::SeqCst);
    }

    pub fn validator(&self) -> CustomValidator {
        CustomValidator::new(
            self.validator_next_flags.clone(),
            self.validator_tick_count.clone(),
        )
    }
}

#[derive(Clone)]
struct ValidatorSlot {
    value: i32,
    flags: u8,
}

#[derive(Clone)]
pub struct CustomValidator {
    slots: Vec<ValidatorSlot>,

    // The invalidation flags used for the next read or write.
    next_flags: Arc<AtomicU8>,
    next_flags_local: u8,

    // last_count is used to quickly invalidate slots without having to
    // iterate over all slots and invalidating manually.
    last_count: usize,

    // tick count is the externally controlled tick number that will
    // invalidate specific caches when it is increased.
    tick_count: Arc<AtomicI32>,
    tick_count_local: i32,
}

impl CustomValidator {
    pub fn new(next_flags: Arc<AtomicU8>, tick_count: Arc<AtomicI32>) -> Self {
        Self {
            slots: vec![],
            next_flags,
            next_flags_local: INVALIDATE_ALWAYS,
            last_count: 0,
            tick_count,
            tick_count_local: -1,
        }
    }
}

impl CacheValidator for CustomValidator {
    // Create a vector containing all slots with a predefined invalid state.
    fn allocate_slots(&mut self, slot_count: usize) {
        self.slots.resize(
            slot_count,
            ValidatorSlot {
                value: -1,
                flags: INVALIDATE_ALWAYS,
            },
        );
    }

    // This function is invoked on every batch of memory operations.
    // This simply updates the internal state and reads the Atomic variables for the upcoming validations.
    fn update_validity(&mut self) {
        self.last_count = self.last_count.wrapping_add(1);
        self.next_flags_local = self.next_flags.load(Ordering::SeqCst);
        self.tick_count_local = self.tick_count.load(Ordering::SeqCst);
    }

    // This simply returns true or false if the slot is valid or not.
    // `last_count` is used here to invalidate slots quickly without requiring to iterate over the entire slot list.
    fn is_slot_valid(&self, slot_id: usize) -> bool {
        match self.slots[slot_id].flags {
            INVALIDATE_ALWAYS => self.slots[slot_id].value == self.last_count as i32,
            INVALIDATE_TICK => self.slots[slot_id].value == self.tick_count_local as i32,
            _ => false,
        }
    }

    // In case the cache is being updates this function marks the slot as being valid.
    fn validate_slot(&mut self, slot_id: usize) {
        match self.next_flags_local {
            INVALIDATE_ALWAYS => self.slots[slot_id].value = self.last_count as i32,
            INVALIDATE_TICK => self.slots[slot_id].value = self.tick_count_local as i32,
            _ => (),
        }

        self.slots[slot_id].flags = self.next_flags_local;
    }

    // In case a slot has to be freed this function resets it to the default values.
    fn invalidate_slot(&mut self, slot_id: usize) {
        self.slots[slot_id].value = -1;
        self.slots[slot_id].flags = INVALIDATE_ALWAYS;
    }
}