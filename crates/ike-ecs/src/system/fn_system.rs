use std::{any::type_name, borrow::Cow};

use crate::{ChangeTick, System, SystemAccess, SystemParam, SystemParamFetch, World};

pub trait SystemParamFn<Params: SystemParam>: Send + Sync + 'static {
    fn access() -> SystemAccess;

    fn run(&mut self, params: &mut Params::Fetch, world: &World, last_change_tick: ChangeTick);
}

macro_rules! impl_system_param_fn {
    () => {
        impl_system_param_fn!(@);
    };
    ($first:ident $(,$name:ident)*) => {
        impl_system_param_fn!($($name),*);
        impl_system_param_fn!(@ $first $(,$name)*);
    };
    (@ $($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        impl<$($name: SystemParam,)* Func: Send + Sync + 'static> SystemParamFn<($($name,)*)> for Func
        where
            for<'a> &'a mut Func:
                FnMut($($name),*) +
                FnMut($(<$name::Fetch as SystemParamFetch>::Item),*),
        {
            fn access() -> SystemAccess {
                let mut access = SystemAccess::default();

                $($name::Fetch::access(&mut access);)*

                access
            }

            fn run(&mut self, params: &mut ($($name::Fetch,)*), world: &World, last_change_tick: ChangeTick) {
                let ($($name,)*) = params;

                fn call_inner<$($name),*>(
                    mut f: impl FnMut($($name),*),
                    $($name: $name),*
                ) {
                    f($($name),*);
                }

                call_inner(self, $($name.get(world, last_change_tick)),*);
            }
        }
    };
}

impl_system_param_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

pub struct FnSystem<F, Params: SystemParam> {
    func: F,
    name: Cow<'static, str>,
    state: Option<Params::Fetch>,
    last_change_tick: ChangeTick,
}

impl<F, Params: SystemParam + 'static> System for FnSystem<F, Params>
where
    F: SystemParamFn<Params>,
{
    fn access(&self) -> SystemAccess {
        F::access()
    }

    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    fn init(&mut self, world: &mut World) {
        self.state = Some(Params::Fetch::init(world));
    }

    fn run(&mut self, world: &World) {
        let last_change_tick = world.increment_change_tick();
        let state = self.state.as_mut().expect("init not run for FnSystem");

        self.func.run(state, world, self.last_change_tick);
        self.last_change_tick = last_change_tick;
    }

    fn apply(&mut self, world: &mut World) {
        let state = self.state.take().expect("init not run for FnSystem");
        state.apply(world);
    }
}

pub trait IntoSystem<Params> {
    type System: System;

    fn system(self) -> Self::System;
}

impl<F, Params> IntoSystem<Params> for F
where
    F: SystemParamFn<Params>,
    Params: SystemParam + 'static,
{
    type System = FnSystem<F, Params>;

    fn system(self) -> Self::System {
        FnSystem {
            func: self,
            name: Cow::Borrowed(type_name::<F>()),
            state: None,
            last_change_tick: 0,
        }
    }
}
