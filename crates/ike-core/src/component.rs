use std::{any::TypeId, collections::HashMap};

use crate::{AnyComponent, CommandQueue, Commands, Node, World, WorldRef};

macro_rules! impl_component {
    ($($name:ident,)*) => {
        #[allow(unused)]
        pub trait Component: AnyComponent {
            $(
                fn $name(&mut self, node: &mut Node, world: &WorldRef) {}
            )*
        }

        #[derive(Default)]
        pub struct ComponentStage {
            run: HashMap<TypeId, fn(&mut Node, &WorldRef)>,
            last_change_tick: u64,
        }

        impl ComponentStage {
            $(
                #[inline]
                fn $name<T: Component>(&mut self) {
                    fn run<T: Component>(node: &mut Node, world: &WorldRef) {
                        let mut component = world.get_component_mut::<T>(&node.entity()).unwrap();

                        component.$name(node, world);
                    }

                    self.run.insert(TypeId::of::<T>(), run::<T>);
                }
            )*

            #[inline]
            pub fn run(&mut self, world: &mut World) {
                let mut command_queue = CommandQueue::default();
                let commands = Commands::new(world.entities(), &mut command_queue);

                let world_ref = WorldRef::new(world, commands, self.last_change_tick);
                self.last_change_tick = world.increment_change_tick();

                for (ty, run) in &self.run {
                    if let Some(storage) = world.entities().storage_raw(ty) {
                        for entity in storage.entities() {
                            let mut node = world_ref.get_node(entity).unwrap();

                            run(&mut node, &world_ref);
                        }
                    }
                }

                command_queue.apply(world);
            }
        }

        #[derive(Default)]
        pub struct ComponentStages {
            $(
                pub $name: ComponentStage,
            )*
        }

        impl ComponentStages {
            #[inline]
            pub fn register<T: Component>(&mut self) {
                $(
                    self.$name.$name::<T>();
                )*
            }
        }
    };
}

impl_component! {
    start,
    pre_update,
    update,
    post_update,
    end,
}
