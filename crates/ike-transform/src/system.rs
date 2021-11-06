use ike_core::*;

use crate::{Children, GlobalTransform, Parent, Transform};

fn update_transform<'a>(
    parent_transform: &Transform,
    parent_changed: bool,
    entity: &Entity,
    changed_query: &mut Query<Entity, Changed<Transform>>,
    query: &mut Query<'a, (&'_ Transform, &'_ mut GlobalTransform, &'_ Children)>,
) {
    if let Some((transform, mut global_transform, children)) = query.get(*entity) {
        let changed = changed_query.get(*entity).is_some();

        let parent_transform = parent_transform * transform;

        if changed || parent_changed {
            *global_transform = parent_transform.clone().into();
        }

        for child in &children.0.clone() {
            let child_changed = changed_query.get(*child).is_some();

            if child_changed {
                update_transform(&parent_transform, changed, child, changed_query, query);
            }
        }
    }
}

pub fn insert_transform_components(
    commands: Commands,
    global_transform: Query<(Entity, &Transform), Without<GlobalTransform>>,
    children: Query<Entity, (With<Transform>, Without<Children>)>,
) {
    for (entity, transform) in global_transform {
        commands.insert_component::<GlobalTransform>(&entity, transform.clone().into());
    }

    for entity in children {
        commands.insert_component(&entity, Children(Vec::new()));
    }
}

pub fn transform_system(world: WorldRef) {
    let mut root = Vec::new();

    for mut children in world.query::<&mut Children>().unwrap() {
        children.0.clear();
    }

    let mut children = world.query::<&mut Children>().unwrap();

    for entity in world
        .query_filter::<Entity, (With<Transform>, Without<Parent>)>()
        .unwrap()
    {
        root.push(entity);
    }

    for (entity, parent) in world
        .query_filter::<(Entity, &Parent), With<Transform>>()
        .unwrap()
    {
        children.get(parent.0).unwrap().0.push(entity);
    }

    drop(children);

    let mut changed_query = world.query_filter::<Entity, Changed<Transform>>().unwrap();

    let mut query = world
        .query::<(&Transform, &mut GlobalTransform, &Children)>()
        .unwrap();

    for entity in root {
        if let Some((transform, mut global_transform, children)) = query.get(entity) {
            let parent_changed = changed_query.get(entity).is_some();

            if parent_changed {
                *global_transform = transform.clone().into();
            }

            let parent_transform = transform.clone();

            for child in &children.0.clone() {
                update_transform(
                    &parent_transform,
                    parent_changed,
                    child,
                    &mut changed_query,
                    &mut query,
                );
            }
        }
    }
}
