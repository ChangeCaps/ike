use std::collections::HashMap;

use ike_ecs::{Entity, ReflectComponent, World};
use ike_reflect::Reflect;
use ike_type::TypeRegistry;
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Scene {
    pub entities: HashMap<Entity, SceneNode>,
}

impl Scene {
    pub fn from_world(world: &World) -> Self {
        let mut this = Self::default();

        let type_registry = world.resource::<TypeRegistry>();

        for entity in world.entities().entities().iter() {
            let mut node = SceneNode::default();

            for registration in type_registry.iter() {
                if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                    if let Some(reflect) = reflect_component.get(world, &entity) {
                        node.components.insert(
                            registration.type_name().to_string(),
                            reflect.clone_dynamic(),
                        );
                    }
                }
            }

            this.entities.insert(entity, node);
        }

        this
    }
}

#[derive(Default, Serialize)]
pub struct SceneNode {
    pub components: HashMap<String, Box<dyn Reflect>>,
}
