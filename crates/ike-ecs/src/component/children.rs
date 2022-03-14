use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{Changed, Commands, Component, Entity, Query, SystemLabel, With, Without};
use ike_reflect::Reflect;

#[derive(Component, Reflect)]
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

#[derive(Component)]
pub struct PreviousParent(Entity);

#[derive(Component, Reflect, Default)]
pub struct Children {
    pub children: Vec<Entity>,
}

impl Deref for Children {
    type Target = Vec<Entity>;

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

#[derive(SystemLabel, Clone, Copy, Debug, Hash)]
pub struct UpdateParentSystem;

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
            children.retain(|entity| *entity != previous_parent.0);
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
                chilren.retain(|child| *child != entity);
            }

            previous_parent.0 = parent.parent;
        } else {
            commands.insert(&entity, PreviousParent(parent.parent));
        }

        if let Some(mut children) = children_query.get_mut(&parent) {
            children.push(entity);
        } else {
            new_children.entry(parent.parent).or_default().push(entity);
        }
    }

    for (entity, children) in new_children {
        commands.insert(&entity, children);
    }
}
