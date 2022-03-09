use ike_ecs::{Changed, Children, Commands, Entity, Parent, Query, With, Without};

use crate::{GlobalTransform, Transform};

pub fn add_global_transform_system(
    commands: Commands,
    query: Query<Entity, (With<Transform>, Without<GlobalTransform>)>,
) {
    for entity in query.iter() {
        commands.insert(&entity, GlobalTransform::default());
    }
}

pub fn transform_propagate_system(
    root_query: Query<
        (Entity, &Transform, Option<&Children>),
        (Without<Parent>, With<GlobalTransform>),
    >,
    transform_query: Query<(Entity, &Transform), (With<Parent>, With<GlobalTransform>)>,
    mut global_transform_query: Query<&mut GlobalTransform>,
    changed_transform_query: Query<Entity, Changed<Transform>>,
    children_query: Query<&Children, (With<Parent>, With<GlobalTransform>)>,
) {
    for (entity, transform, children) in root_query.iter() {
        let mut changed = false;

        let mut global_transform = global_transform_query.get_mut(&entity).unwrap();

        if changed_transform_query.contains(&entity) {
            *global_transform = GlobalTransform::new(*transform);
            changed = true;
        }

        let global_transform = global_transform.clone();

        if let Some(children) = children {
            for child in children.iter() {
                propagate_recursive(
                    &global_transform,
                    &changed_transform_query,
                    &transform_query,
                    &mut global_transform_query,
                    &children_query,
                    *child,
                    changed,
                );
            }
        }
    }
}

fn propagate_recursive(
    parent: &GlobalTransform,
    changed_transform_query: &Query<Entity, Changed<Transform>>,
    transform_query: &Query<(Entity, &Transform), (With<Parent>, With<GlobalTransform>)>,
    global_transform_query: &mut Query<&mut GlobalTransform>,
    children_query: &Query<&Children, (With<Parent>, With<GlobalTransform>)>,
    entity: Entity,
    mut changed: bool,
) {
    changed |= children_query.contains(&entity);

    let global_transform = if let Some((_entity, transform)) = transform_query.get(&entity) {
        let mut global_transform = global_transform_query.get_mut(&entity).unwrap();

        if changed {
            *global_transform = GlobalTransform::new(parent.transform() * *transform);
        }

        global_transform.clone()
    } else {
        return;
    };

    if let Some(children) = children_query.get(&entity) {
        for child in children.iter() {
            propagate_recursive(
                &global_transform,
                changed_transform_query,
                transform_query,
                global_transform_query,
                children_query,
                *child,
                changed,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use ike_app::CoreStage;
    use ike_ecs::{update_parent_system, Schedule, World};
    use ike_math::Vec3;

    use super::*;

    #[test]
    fn propagate_transform() {
        let mut world = World::new();

        let mut schedule = Schedule::new();
        schedule
            .add_system_to_stage(update_parent_system, CoreStage::Update)
            .unwrap();
        schedule
            .add_system_to_stage(transform_propagate_system, CoreStage::Update)
            .unwrap();

        let mut children = Vec::new();

        world.commands(|commands| {
            commands
                .spawn()
                .insert(Transform::from_xyz(2.0, 1.0, 0.0))
                .insert(GlobalTransform::IDENTITY)
                .with_children(|parent| {
                    let entity = parent
                        .spawn()
                        .insert(Transform::from_xyz(1.0, -1.0, 1.0))
                        .insert(GlobalTransform::IDENTITY)
                        .entity();

                    children.push(entity);

                    let entity = parent
                        .spawn()
                        .insert(Transform::from_xyz(2.0, 3.0, -1.0))
                        .insert(GlobalTransform::IDENTITY)
                        .entity();

                    children.push(entity);
                });
        });

        schedule.execute(&mut world);

        assert_eq!(
            Vec3::new(3.0, 0.0, 1.0),
            world
                .component::<GlobalTransform>(&children[0])
                .unwrap()
                .translation
        );
        assert_eq!(
            Vec3::new(4.0, 4.0, -1.0),
            world
                .component::<GlobalTransform>(&children[1])
                .unwrap()
                .translation
        );
    }
}
