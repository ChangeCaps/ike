use std::sync::atomic::{AtomicU64, Ordering};

pub type ChangeTick = u64;

pub struct ChangeTicks {
    last_change_tick: ChangeTick,
    change_tick: ChangeTick,
}

impl ChangeTicks {
    pub const fn new(last_change_tick: ChangeTick, change_tick: ChangeTick) -> Self {
        Self {
            last_change_tick,
            change_tick,
        }
    }

    pub const fn last_change_tick(&self) -> ChangeTick {
        self.last_change_tick
    }

    pub const fn change_tick(&self) -> ChangeTick {
        self.change_tick
    }

    pub const fn delta(&self) -> ChangeTick {
        self.change_tick().wrapping_sub(self.last_change_tick())
    }

    pub const fn is_changed(&self, component_tick: ChangeTick) -> bool {
        let component_delta = self.change_tick().wrapping_sub(component_tick);

        component_delta < self.delta()
    }
}

#[derive(Debug)]
pub struct ComponentTicks {
    added: AtomicU64,
    changed: AtomicU64,
}

impl ComponentTicks {
    pub const fn new(change_tick: ChangeTick) -> Self {
        Self {
            added: AtomicU64::new(change_tick),
            changed: AtomicU64::new(change_tick),
        }
    }

    pub fn added(&self) -> ChangeTick {
        self.added.load(Ordering::Acquire)
    }

    pub fn changed(&self) -> ChangeTick {
        self.changed.load(Ordering::Acquire)
    }

    pub fn changed_raw(&self) -> &AtomicU64 {
        &self.changed
    }

    pub fn change_ticks(&self, change_tick: ChangeTick) {
        if let Some(new_tick) = check_tick(self.added(), change_tick) {
            self.added.store(new_tick, Ordering::Release);
        }

        if let Some(new_tick) = check_tick(self.changed(), change_tick) {
            self.set_changed(new_tick);
        }
    }

    pub fn is_changed(&self, change_ticks: &ChangeTicks) -> bool {
        change_ticks.is_changed(self.changed())
    }

    pub fn is_added(&self, change_ticks: &ChangeTicks) -> bool {
        change_ticks.is_changed(self.added())
    }

    pub fn set_changed(&self, change_tick: ChangeTick) {
        self.changed.store(change_tick, Ordering::Release);
    }
}

fn check_tick(last_change_tick: ChangeTick, change_tick: ChangeTick) -> Option<ChangeTick> {
    let tick_delta = change_tick.wrapping_sub(last_change_tick);
    const MAX_DELTA: u64 = (u64::MAX / 4) * 3;

    if tick_delta > MAX_DELTA {
        Some(change_tick.wrapping_sub(MAX_DELTA))
    } else {
        None
    }
}
