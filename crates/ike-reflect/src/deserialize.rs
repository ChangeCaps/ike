use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};

use crate::{type_field, DynamicStruct, Reflect, ReflectDeserialize, TypeRegistry};

pub struct ReflectDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a> ReflectDeserializer<'a> {
    #[inline]
    pub fn new(type_registry: &'a TypeRegistry) -> Self {
        Self { type_registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct ReflectVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ReflectVisitor<'a> {
    type Value = Box<dyn Reflect>;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("reflect value")
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut type_name = None;

        while let Some(key) = map.next_key()? {
            match key {
                type_field::TYPE => {
                    type_name = Some(map.next_value::<&str>()?);
                }
                type_field::STRUCT => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| Error::missing_field(type_field::TYPE))?;

                    let mut dynamic_struct = map.next_value_seed(StructDeserializer {
                        type_registry: self.type_registry,
                    })?;

                    dynamic_struct.set_name(type_name);

                    return Ok(Box::new(dynamic_struct));
                }
                type_field::VALUE => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| Error::missing_field(type_field::TYPE))?;

                    if let Some(registration) = self.type_registry.get_name(type_name) {
                        if let Some(deserialize) = registration.data::<ReflectDeserialize>() {
                            return map.next_value_seed(ValueDeserializer { deserialize });
                        } else {
                            return Err(Error::custom(format!(
                                "deserialize not registered for '{}'",
                                type_name
                            )));
                        }
                    } else {
                        return Err(Error::custom(format!(
                            "registration not found for '{}'",
                            type_name
                        )));
                    }
                }
                _ => return Err(Error::unknown_field(key, &[])),
            }
        }

        Err(Error::custom("expected reflect value"))
    }
}

struct StructDeserializer<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for StructDeserializer<'a> {
    type Value = DynamicStruct;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(StructVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct StructVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = DynamicStruct;

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("reflect struct")
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut dynamic_struct = DynamicStruct::default();

        while let Some(key) = map.next_key()? {
            let value = map.next_value_seed(ReflectDeserializer {
                type_registry: self.type_registry,
            })?;

            dynamic_struct.insert_boxed(key, value);
        }

        Ok(dynamic_struct)
    }
}

struct ValueDeserializer<'a> {
    deserialize: &'a ReflectDeserialize,
}

impl<'a, 'de> DeserializeSeed<'de> for ValueDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        self.deserialize
            .deserialize(&mut erased)
            .map_err(<<D as serde::Deserializer<'de>>::Error as serde::de::Error>::custom)
    }
}
