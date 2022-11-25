use std::{any::TypeId, ops::Deref};

use downcast_rs::{impl_downcast, Downcast};

use crate::App;

pub trait PluginId: Downcast + Send + Sync {
    fn eq(&self, other: &dyn PluginId) -> bool;
}

impl_downcast!(PluginId);

impl PartialEq for dyn PluginId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl Eq for dyn PluginId {}

impl<T: PartialEq + Send + Sync + 'static> PluginId for T {
    fn eq(&self, other: &dyn PluginId) -> bool {
        if let Some(other) = other.downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }
}

pub trait Plugin: Downcast + Send + Sync + 'static {
    #[inline]
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn build(&self, app: &mut App);

    #[inline]
    fn dependencies(&self, _plugins: &mut Plugins) {}

    #[inline]
    fn id(&self) -> Box<dyn PluginId> {
        Box::new(TypeId::of::<Self>())
    }
}

impl_downcast!(Plugin);

#[derive(Default)]
pub struct Plugins {
    queue: Vec<Box<dyn PluginId>>,
    plugins: Vec<(Box<dyn PluginId>, Box<dyn Plugin>)>,
    built: usize,
}

impl Plugins {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    #[inline]
    pub fn add<T: Plugin>(&mut self, plugin: T) {
        let id = plugin.id();

        if self.contains(id.as_ref()) {
            return;
        }

        if self.is_queued(id.as_ref()) {
            panic!("Circular dependency detected");
        }

        self.queue.push(id);

        plugin.dependencies(self);

        let id = self.queue.pop().unwrap();
        self.plugins.push((id, Box::new(plugin)));
    }

    #[inline]
    pub fn is_queued(&self, plugin_id: &dyn PluginId) -> bool {
        self.queue.iter().any(|id| (*id).deref() == plugin_id)
    }

    #[inline]
    pub fn contains(&self, plugin_id: &dyn PluginId) -> bool {
        (self.plugins.iter()).any(|(id, _)| (*id).deref() == plugin_id)
    }

    #[inline]
    pub fn get<T: Plugin>(&self) -> Option<&T> {
        let plugin_id = TypeId::of::<T>();

        self.plugins
            .iter()
            .find(|(id, _)| (*id).deref() == &plugin_id as &dyn PluginId)
            .map(|(_, plugin)| plugin.downcast_ref::<T>().unwrap())
    }

    #[inline]
    pub fn build(&mut self, app: &mut App) {
        for (_, plugin) in self.plugins[self.built..].iter() {
            plugin.build(app);
        }

        self.built = self.plugins.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn selv_dependency() {
        struct PluginA;

        impl Plugin for PluginA {
            fn build(&self, _app: &mut App) {}

            fn dependencies(&self, plugins: &mut Plugins) {
                plugins.add(PluginA);
            }
        }

        let mut plugins = Plugins::new();

        plugins.add(PluginA);
    }

    #[test]
    #[should_panic]
    fn cyclic_dependency() {
        struct PluginA;

        impl Plugin for PluginA {
            fn build(&self, _app: &mut App) {}

            fn dependencies(&self, plugins: &mut Plugins) {
                plugins.add(PluginB);
            }
        }

        struct PluginB;

        impl Plugin for PluginB {
            fn build(&self, _app: &mut App) {}

            fn dependencies(&self, plugins: &mut Plugins) {
                plugins.add(PluginA);
            }
        }

        let mut plugins = Plugins::new();

        plugins.add(PluginA);
        plugins.add(PluginB);
    }

    #[test]
    fn order() {
        struct PluginA;

        impl Plugin for PluginA {
            fn build(&self, app: &mut App) {
                app.insert_resource(5i32);
            }
        }

        struct PluginB;

        impl Plugin for PluginB {
            fn build(&self, app: &mut App) {
                assert_eq!(app.remove_resource::<i32>(), Some(5));
            }

            fn dependencies(&self, plugins: &mut Plugins) {
                plugins.add(PluginA);
            }
        }

        let mut plugins = Plugins::new();
        plugins.add(PluginB);

        let mut app = App::new();
        plugins.build(&mut app);
    }
}
