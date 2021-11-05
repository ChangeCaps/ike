use ike_core::*;

use crate::{Children, GlobalTransform, Parent, Transform};

fn update_transform<'a>(
    parent_transform: &Transform,
    parent_changed: bool,
    entity: &Entity,
    changed_query: &mut QueryMut<(), Changed<Transform>>,
    query: &mut QueryMut<'a, (&'_ Transform, &'_ mut GlobalTransform, &'_ Children)>,
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

pub struct TransformSystem;

impl ExclusiveSystem for TransformSystem {
    fn run(&mut self, world: &mut World) {
        // give all entities global transform and children
        for entity in world
            .query::<Entity, (With<Transform>, Without<GlobalTransform>)>()
            .unwrap()
        {
            world.queue_insert(entity, GlobalTransform::IDENTITY);
        }

        for entity in world
            .query::<Entity, (With<Transform>, Without<Children>)>()
            .unwrap()
        {
            world.queue_insert(entity, Children(Vec::new()));
        }

        world.dequeue();

        let mut root = Vec::new();

        for mut children in world.query::<&mut Children, ()>().unwrap() {
            children.0.clear();
        }

        let mut children = world.query::<&mut Children, ()>().unwrap(); 

        for entity in world
            .query::<Entity, (With<Transform>, Without<Parent>)>()
            .unwrap()
        {
            root.push(entity);
        }

        for (entity, parent) in world
            .query::<(Entity, &Parent), With<Transform>>()
            .unwrap()
        {
            children.get(parent.0).unwrap().0.push(entity);
        }

        drop(children);

        let mut changed_query = world.query::<(), Changed<Transform>>().unwrap();

        let mut query = world
            .query::<(&Transform, &mut GlobalTransform, &Children), ()>()
            .unwrap();


        for entity in root {
            if let Some((transform, mut global_transform, children)) = query.get(entity) {
                let parent_changed = changed_query.get(entity).is_some();

                if parent_changed {
                    *global_transform = transform.clone().into();
                }

                let parent_transform = transform.clone();

                for child in &children.0.clone() { 
                    update_transform(&parent_transform, parent_changed, child, &mut changed_query, &mut query);
                }
            }
        }
    }
}
