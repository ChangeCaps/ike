use std::{
    collections::{BTreeSet, HashMap},
    ops::{Deref, DerefMut},
};

use crate::{Changed, Commands, Component, Entity, Query, SparseStorage, With, Without};

pub struct Parent {
    pub parent: Entity,
}

impl Parent {
    pub const fn new(parent: Entity) -> Self {
        Self { parent }
    }
}

impl Deref for Parent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl DerefMut for Parent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent
    }
}

impl Component for Parent {
    type Storage = SparseStorage;
}

pub struct PreviousParent(Entity);

impl Component for PreviousParent {
    type Storage = SparseStorage;
}

#[derive(Default)]
pub struct Children {
    pub children: BTreeSet<Entity>,
}

impl Deref for Children {
    type Target = BTreeSet<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl DerefMut for Children {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl Children {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Children {
    type Storage = SparseStorage;
}

pub fn update_parent_system(
    commands: Commands,
    removed_parent_query: Query<Entity, (Without<Parent>, With<PreviousParent>)>,
    parent_query: Query<(Entity, &Parent), Changed<Parent>>,
    mut previous_parent_query: Query<&mut PreviousParent>,
    mut children_query: Query<&mut Children>,
) {
    for entity in removed_parent_query.iter() {
        if let Some(mut children) = children_query.get_mut(&entity) {
            let previous_parent = previous_parent_query.get(&entity).unwrap();
            children.remove(&previous_parent.0);
            commands.remove::<PreviousParent>(&entity);
        }
    }

    let mut new_children = HashMap::<Entity, Children>::new();

    for (entity, parent) in parent_query.iter() {
        if let Some(mut previous_parent) = previous_parent_query.get_mut(&entity) {
            if previous_parent.0 == parent.parent {
                continue;
            }

            if let Some(mut chilren) = children_query.get_mut(&previous_parent.0) {
                chilren.remove(&entity);
            }

            previous_parent.0 = parent.parent;
        } else {
            commands.insert(&entity, PreviousParent(parent.parent));
        }

        if let Some(mut children) = children_query.get_mut(&parent) {
            children.insert(entity);
        } else {
            new_children
                .entry(parent.parent)
                .or_default()
                .insert(entity);
        }
    }

    for (entity, children) in new_children {
        commands.insert(&entity, children);
    }
}
