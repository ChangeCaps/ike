use crate::{Commands, Component, Entity, Parent};

pub struct SpawnEntity<'w, 's> {
    commands: &'s Commands<'w, 's>,
    entity: Entity,
}

impl<'w, 's> SpawnEntity<'w, 's> {
    pub fn new(commands: &'s Commands<'w, 's>, entity: Entity) -> Self {
        Self { entity, commands }
    }

    pub fn insert<T: Component>(&self, component: T) -> &Self {
        self.commands.insert(&self.entity, component);
        self
    }

    pub fn with_children(&self, f: impl FnOnce(SpawnChildren)) -> &Self {
        let spawn_children = SpawnChildren {
            commands: self.commands,
            parent: self.entity,
        };

        f(spawn_children);

        self
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}

pub struct SpawnChildren<'w, 's> {
    commands: &'s Commands<'w, 's>,
    parent: Entity,
}

impl<'w, 's> SpawnChildren<'w, 's> {
    pub fn spawn(&self) -> SpawnEntity<'_, '_> {
        let spawn = self.commands.spawn();
        spawn.insert(Parent::new(self.parent));
        spawn
    }
}
