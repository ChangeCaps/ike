use std::{any::TypeId, ops::Deref};

use downcast_rs::{impl_downcast, Downcast};

use crate::App;

/// An id for a [`Plugin`]. By default, this is the [`TypeId`] of the plugin.
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

impl<T: PartialEq + Sync + Send + 'static> PluginId for T {
    #[inline]
    fn eq(&self, other: &dyn PluginId) -> bool {
        if let Some(other) = other.downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }
}

/// A plugin that can be added to an [`App`].
///
/// Plugin types should only contain data required to configure the plugin. The actual setup should
/// be done in [`Plugin::build`].
pub trait Plugin: Downcast + Send + Sync + 'static {
    #[inline]
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Builds the plugin.
    ///
    /// This is always called after all dependencies have been built.
    fn build(&self, app: &mut App);

    /// Adds dependencies of this plugin.
    ///
    /// Plugins do not support circular dependencies.
    #[inline]
    fn dependencies(&self, _plugins: &mut Plugins) {}

    /// Returns the [`PluginId`] for this plugin.
    ///
    /// By default, this is the [`DefaultPluginId`] of the plugin type.
    #[inline]
    fn id(&self) -> Box<dyn PluginId> {
        Box::new(TypeId::of::<Self>())
    }
}

impl_downcast!(Plugin);

impl std::fmt::Debug for dyn Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl dyn Plugin {
    /// Returns an [`ExactSizeIterator`] over the dependencies of this plugin.
    #[inline]
    pub fn list_dependencies(&self) -> impl ExactSizeIterator<Item = Box<dyn Plugin>> {
        let mut plugins = Plugins::new();
        self.dependencies(&mut plugins);
        plugins.plugins.into_iter().map(|(_, p)| p)
    }
}

/// A collection of [`Plugin`]s.
///
/// This should usually not be used directly. Instead, use [`App::add_plugin`].
#[derive(Default)]
pub struct Plugins {
    dependants: Vec<(Box<dyn PluginId>, String)>,
    plugins: Vec<(Box<dyn PluginId>, Box<dyn Plugin>)>,
    built: usize,
}

impl Plugins {
    /// Creates a empty [`Plugins`].
    #[inline]
    pub const fn new() -> Self {
        Self {
            dependants: Vec::new(),
            plugins: Vec::new(),
            built: 0,
        }
    }

    /// Returns the number of plugins.
    #[inline]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns the number of plugins that have been built.
    #[inline]
    pub const fn built(&self) -> usize {
        self.built
    }

    /// Adds a plugin and it's [`Plugin::dependencies`].
    ///
    /// # Panics
    /// - Panics if the plugin contains a circular dependency.
    #[inline]
    pub fn add<T: Plugin>(&mut self, plugin: T) {
        let id = plugin.id();

        if self.contains(id.as_ref()) {
            return;
        }

        if self.is_dependency(id.as_ref()) {
            let names = (self.dependants.iter().map(|(_, name)| name)).collect::<Vec<_>>();
            panic!(
                "Circular dependency detected in {}, {:?}",
                plugin.name(),
                names
            );
        }

        self.dependants.push((id, plugin.name().to_string()));

        plugin.dependencies(self);

        let (id, _) = self.dependants.pop().unwrap();
        self.plugins.push((id, Box::new(plugin)));
    }

    #[inline]
    fn is_dependency(&self, plugin_id: &dyn PluginId) -> bool {
        self.dependants
            .iter()
            .any(|(id, _)| (*id).deref() == plugin_id)
    }

    /// Returns `true` if the plugin has already been added.
    #[inline]
    pub fn contains(&self, plugin_id: &dyn PluginId) -> bool {
        (self.plugins.iter()).any(|(id, _)| (*id).deref() == plugin_id)
    }

    /// Gets a plugin by [`TypeId`].
    ///
    /// If the plugin has a custom [`Plugin::id`], use [`Plugins::get_by_id`] instead.
    #[inline]
    pub fn get<T: Plugin>(&self) -> Option<&T> {
        let plugin_id = TypeId::of::<T>();

        self.get_by_id(&plugin_id)?.downcast_ref()
    }

    /// Gets a plugin by it's [`PluginId`].
    #[inline]
    pub fn get_by_id(&self, plugin_id: &dyn PluginId) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|(id, _)| (*id).deref() == plugin_id)
            .map(|(_, plugin)| plugin.as_ref())
    }

    /// Builds all plugins that haven't been built yet.
    #[inline]
    pub fn build(&mut self, app: &mut App) {
        for (_, plugin) in self.plugins[self.built..].iter() {
            plugin.build(app);
        }

        self.built = self.plugins.len();
    }

    /// Builds all plugins.
    #[inline]
    pub fn build_all(&mut self, app: &mut App) {
        for (_, plugin) in self.plugins.iter() {
            plugin.build(app);
        }
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
