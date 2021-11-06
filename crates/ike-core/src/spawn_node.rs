use crate::{AnyComponent, Commands, Entity};

pub struct SpawnNode<'w, 's> {
    entity: Entity,
    commands: &'w Commands<'w, 's>,
}

impl<'w, 's> SpawnNode<'w, 's> {
    #[inline]
    pub fn new(entity: Entity, commands: &'w Commands<'w, 's>) -> Self {
        Self { entity, commands }
    }

    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub fn insert<T: AnyComponent>(&self, component: T) {
        self.commands.insert_component(&self.entity, component);
    }
}
