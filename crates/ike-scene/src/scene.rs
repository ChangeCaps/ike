use std::{any::TypeId, collections::HashMap};

use ike_core::{Commands, Entity, WorldRef};
use ike_reflect::{Reflect, ReflectComponent, ReflectDeserializer, TypeRegistry};
use serde::{
    de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize,
};

#[derive(Default)]
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
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
        type_registry: &TypeRegistry,
    ) -> Result<Self, D::Error> {
        SceneDeserializer::new(type_registry).deserialize(deserializer)
    }

    #[inline]
    pub fn add(&mut self, node: SceneNode) -> Entity {
        let entity = Entity::from_raw(self.next_entity, 0);
        self.next_entity += 1;

        self.entities.insert(entity, node);

        entity
    }

    #[inline]
    pub fn insert(&mut self, entity: Entity, node: SceneNode) {
        self.entities.insert(entity, node);
        self.next_entity = self.next_entity.max(entity.idx() + 1);
    }

    #[inline]
    pub fn from_world(world: &WorldRef, type_registry: &TypeRegistry) -> Self {
        let mut scene = Self::new();

        for entity in world.entities().entities() {
            let name = world
                .get_node(entity)
                .map_or(String::new(), |node| node.name().clone());

            let mut node = SceneNode::new(name);

            for (type_id, registration) in type_registry.iter() {
                if let Some(reflect_component) =
                    unsafe { registration.data_named::<ReflectComponent>() }
                {
                    if let Some(storage) = world.entities().storage_raw(type_id) {
                        if let Some(component) =
                            unsafe { reflect_component.from_storage_mut(storage, entity) }
                        {
                            node.insert_boxed(*type_id, component.clone_value());
                        }
                    }
                }
            }

            scene.insert(*entity, node);
        }

        scene
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

    #[inline]
    pub fn insert_boxed(&mut self, type_id: TypeId, component: Box<dyn Reflect>) {
        self.components.insert(type_id, component);
    }
}

impl Serialize for Scene {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.entities.serialize(serializer)
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
        let mut state = serializer.serialize_seq(Some(self.0.len()))?;

        for value in self.0.values() {
            state.serialize_element(value)?;
        }

        state.end()
    }
}

pub struct SceneDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a> SceneDeserializer<'a> {
    #[inline]
    pub fn new(type_registry: &'a TypeRegistry) -> Self {
        Self { type_registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = Scene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(SceneVisitor {
            type_registry: self.type_registry,
        })
    }
}

pub struct SceneVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneVisitor<'a> {
    type Value = Scene;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("scene")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entities = HashMap::new();

        while let Some(key) = map.next_key::<Entity>()? {
            let value = map.next_value_seed(SceneNodeDeserializer {
                type_registry: self.type_registry,
            })?;

            entities.insert(key, value);
        }

        let next_entity = entities
            .keys()
            .map(|entity| entity.idx() + 1)
            .max()
            .unwrap_or(0);

        Ok(Scene {
            entities,
            next_entity,
        })
    }
}

pub struct SceneNodeDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneNodeDeserializer<'a> {
    type Value = SceneNode;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "SceneNode",
            &["name", "component"],
            SceneNodeVisitor {
                type_registry: self.type_registry,
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    Name,
    Components,
}

pub struct SceneNodeVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneNodeVisitor<'a> {
    type Value = SceneNode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("scene node")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name = None;
        let mut components = None;

        while let Some(key) = map.next_key::<Field>()? {
            match key {
                Field::Name => {
                    name = Some(map.next_value::<String>()?);
                }
                Field::Components => {
                    components = Some(map.next_value_seed(ComponentsDeserializer {
                        type_registry: self.type_registry,
                    })?);
                }
            }
        }

        let name = name.ok_or_else(|| Error::missing_field("name"))?;
        let components = components.ok_or_else(|| Error::missing_field("components"))?;

        Ok(SceneNode { name, components })
    }
}

pub struct ComponentsDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentsDeserializer<'a> {
    type Value = HashMap<TypeId, Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ComponentsVisitor {
            type_registry: self.type_registry,
        })
    }
}

pub struct ComponentsVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ComponentsVisitor<'a> {
    type Value = HashMap<TypeId, Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("components")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut components = HashMap::new();

        while let Some(component) =
            seq.next_element_seed(ReflectDeserializer::new(self.type_registry))?
        {
            if let Some(registration) = self.type_registry.get_name(component.type_name()) {
                components.insert(registration.type_id(), component);
            }
        }

        Ok(components)
    }
}
