use std::{any::TypeId, collections::HashMap};

use ike_core::{Commands, Entity};
use ike_reflect::{Reflect, ReflectComponent, TypeRegistry};
use serde::{
    ser::{SerializeMap, SerializeStruct},
    Serialize,
};

#[derive(Default, Serialize)]
pub struct Scene {
    pub entities: HashMap<Entity, SceneNode>,
    pub next_entity: u64,
}

impl Scene {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert(&mut self, node: SceneNode) -> Entity {
        let entity = Entity::from_raw(self.next_entity, 0);
        self.next_entity += 1;

        self.entities.insert(entity, node);

        entity
    }

    #[inline]
    pub fn spawn(&self, commands: &Commands, type_registry: &TypeRegistry) {
        let entity_map = self
            .entities
            .iter()
            .map(|(scene_entity, scene_node)| {
                let world_node = commands.spawn_node(&scene_node.name);

                (*scene_entity, world_node.entity())
            })
            .collect::<HashMap<_, _>>();

        for (scene_entity, scene_node) in self.entities.iter() {
            for (type_id, component) in scene_node.components.iter() {
                let world_entity = &entity_map[&scene_entity];
                let component = component.map_values(|scene_entity: &mut Entity| {
                    *scene_entity = entity_map[scene_entity]
                });

                if let Some(registration) = type_registry.get(type_id) {
                    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
                        let _ =
                            reflect_component.insert(world_entity, component.as_ref(), commands);
                    }
                }
            }
        }
    }
}

pub struct SceneNode {
    pub name: String,
    pub components: HashMap<TypeId, Box<dyn Reflect>>,
}

impl SceneNode {
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            components: HashMap::default(),
        }
    }

    #[inline]
    pub fn insert<T: Reflect>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }
}

impl Serialize for SceneNode {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SceneNode", 2)?;

        state.serialize_field("name", &self.name)?;
        state.serialize_field("components", &ComponentsSerializer(&self.components))?;

        state.end()
    }
}

pub struct ComponentsSerializer<'a>(&'a HashMap<TypeId, Box<dyn Reflect>>);

impl<'a> Serialize for ComponentsSerializer<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.0.len()))?;

        for value in self.0.values() {
            state.serialize_entry(value.type_name(), value)?;
        }

        state.end()
    }
}
