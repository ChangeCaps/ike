use std::{any::type_name, borrow::Cow};

use crate::{ChangeTick, World};

pub trait ExclusiveSystem: Send + Sync + 'static {
    fn name(&self) -> &Cow<'static, str>;

    fn run(&mut self, world: &mut World);
}

pub struct ExclusiveSystemFn<F> {
    func: F,
    name: Cow<'static, str>,
    last_change_tick: ChangeTick,
}

impl<F> ExclusiveSystem for ExclusiveSystemFn<F>
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    fn run(&mut self, world: &mut World) {
        let saved_change_tick = world.last_change_tick();
        world.set_last_change_tick(self.last_change_tick);

        (self.func)(world);

        self.last_change_tick = world.change_tick();
        world.increment_change_tick();

        world.set_last_change_tick(saved_change_tick);
    }
}

pub trait IntoExclusiveSystem<System> {
    fn exclusive_system(self) -> System;
}

impl<F> IntoExclusiveSystem<ExclusiveSystemFn<F>> for F
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn exclusive_system(self) -> ExclusiveSystemFn<F> {
        ExclusiveSystemFn {
            func: self,
            name: Cow::Borrowed(type_name::<F>()),
            last_change_tick: 0,
        }
    }
}
