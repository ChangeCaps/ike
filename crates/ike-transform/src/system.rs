use ike_core::*;

use crate::{Children, GlobalTransform, Parent, Transform};

fn update_transform<'a>(
    parent_transform: &Transform,
    entity: &Entity,
    query: &mut QueryMut<'a, (&'_ Transform, &'_ mut GlobalTransform, &'_ Children)>,
) {
    let (transform, global_transform, children) = query.get(*entity).unwrap();

    let parent_transform = parent_transform * transform;
    *global_transform = parent_transform.clone().into();

    for child in &children.0.clone() {
        update_transform(&parent_transform, child, query);
    }
}

pub struct TransformSystem;

impl ExclusiveSystem for TransformSystem {
    fn run(&mut self, world: &mut World) {
        // give all entities global transform and children
        for entity in world
            .query::<Entity>()
            .unwrap()
            .with::<Transform>()
            .without::<GlobalTransform>()
        {
            world.queue_insert(entity, GlobalTransform::IDENTITY);
        }

        for entity in world
            .query::<Entity>()
            .unwrap()
            .with::<Transform>()
            .without::<Children>()
        {
            world.queue_insert(entity, Children(Vec::new()));
        }

        world.dequeue();

        let mut root = Vec::new();

        for children in world.query::<&mut Children>().unwrap() {
            children.0.clear();
        }

        let mut children = world.query::<&mut Children>().unwrap();

        for entity in world
            .query::<Entity>()
            .unwrap()
            .without::<Parent>()
            .with::<Transform>()
        {
            root.push(entity);
        }

        for (entity, parent) in world
            .query::<(Entity, &Parent)>()
            .unwrap()
            .with::<Transform>()
        {
            children.get(parent.0).unwrap().0.push(entity);
        }

        drop(children);

        let mut query = world
            .query::<(&Transform, &mut GlobalTransform, &Children)>()
            .unwrap();

        for entity in root {
            let (transform, global_transform, children) = query.get(entity).unwrap();

            *global_transform = transform.clone().into();
            let parent_transform = transform.clone();

            for child in &children.0.clone() {
                update_transform(&parent_transform, child, &mut query);
            }
        }
    }
}
