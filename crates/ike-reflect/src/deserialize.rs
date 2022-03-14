use ike_type::{FromType, TypeRegistry};
use serde::{
    de::{DeserializeOwned, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    field_type, DynamicEnum, DynamicList, DynamicMap, DynamicSet, DynamicStruct, DynamicTuple,
    DynamicVariant, Reflect,
};

#[derive(Clone)]
pub struct ReflectDeserialize {
    deserialize:
        fn(&mut dyn erased_serde::Deserializer) -> Result<Box<dyn Reflect>, erased_serde::Error>,
}

impl ReflectDeserialize {
    pub fn new<T: Reflect + DeserializeOwned>() -> Self {
        Self {
            deserialize: |deserializer| Ok(Box::new(T::deserialize(deserializer)?)),
        }
    }

    pub fn deserialize(
        &self,
        deserializer: &mut dyn erased_serde::Deserializer,
    ) -> Result<Box<dyn Reflect>, erased_serde::Error> {
        (self.deserialize)(deserializer)
    }
}

impl<T: Reflect + DeserializeOwned> FromType<T> for ReflectDeserialize {
    fn from_type() -> Self {
        Self::new::<T>()
    }
}

pub struct ReflectDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a> ReflectDeserializer<'a> {
    pub fn new(type_registry: &'a TypeRegistry) -> Self {
        Self { type_registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectVisitor(self.type_registry))
    }
}

struct ReflectVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for ReflectVisitor<'a> {
    type Value = Box<dyn Reflect>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("reflect")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut type_name = None;

        while let Some(key) = map.next_key()? {
            match key {
                field_type::TYPE => {
                    type_name = Some(map.next_value::<String>()?);
                }
                field_type::TUPLE => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(TupleDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::STRUCT => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(StructDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::ENUM => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(EnumDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::LIST => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(ListDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::SET => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(SetDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::MAP => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    let mut value = map.next_value_seed(MapDeserializer(self.0))?;

                    value.set_name(type_name);

                    return Ok(Box::new(value));
                }
                field_type::VALUE => {
                    let type_name =
                        type_name.ok_or_else(|| Error::missing_field(field_type::TYPE))?;

                    if let Some(registration) = self.0.get_name(&type_name) {
                        if let Some(deserialize) = registration.data::<ReflectDeserialize>() {
                            return map.next_value_seed(ValueDeserializer(deserialize));
                        } else {
                            return Err(Error::custom(format!(
                                "reflect deserialize not registered for: {}",
                                type_name
                            )));
                        }
                    } else {
                        return Err(Error::custom(format!(
                            "registration not found for: {}",
                            type_name
                        )));
                    }
                }
                _ => {
                    return Err(Error::unknown_field(
                        key,
                        &[
                            field_type::TYPE,
                            field_type::TUPLE,
                            field_type::STRUCT,
                            field_type::ENUM,
                            field_type::LIST,
                            field_type::SET,
                            field_type::MAP,
                            field_type::VALUE,
                        ],
                    ))
                }
            }
        }

        Err(Error::custom("expected reflect"))
    }
}

struct TupleDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for TupleDeserializer<'a> {
    type Value = DynamicTuple;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TupleVisitor(self.0))
    }
}

struct TupleVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for TupleVisitor<'a> {
    type Value = DynamicTuple;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut value = DynamicTuple::default();

        while let Some(field) = seq.next_element_seed(ReflectDeserializer::new(self.0))? {
            value.push_boxed(field);
        }

        Ok(value)
    }
}

struct StructDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for StructDeserializer<'a> {
    type Value = DynamicStruct;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StructVisitor(self.0))
    }
}

struct StructVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = DynamicStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut value = DynamicStruct::default();

        while let Some(name) = map.next_key::<String>()? {
            let field = map.next_value_seed(ReflectDeserializer::new(self.0))?;

            value.push_boxed(name, field);
        }

        Ok(value)
    }
}

struct EnumDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for EnumDeserializer<'a> {
    type Value = DynamicEnum;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EnumVisitor(self.0))
    }
}

struct EnumVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for EnumVisitor<'a> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut variant_name = None;

        while let Some(key) = map.next_key()? {
            match key {
                field_type::VARIANT => {
                    variant_name = Some(map.next_value::<String>()?);
                }
                field_type::TUPLE => {
                    let variant_name =
                        variant_name.ok_or_else(|| Error::missing_field(field_type::VARIANT))?;

                    let value = map.next_value_seed(TupleDeserializer(self.0))?;

                    return Ok(DynamicEnum::new(
                        variant_name,
                        DynamicVariant::Tuple(Box::new(value)),
                    ));
                }
                field_type::STRUCT => {
                    let variant_name =
                        variant_name.ok_or_else(|| Error::missing_field(field_type::VARIANT))?;

                    let value = map.next_value_seed(StructDeserializer(self.0))?;

                    return Ok(DynamicEnum::new(
                        variant_name,
                        DynamicVariant::Struct(Box::new(value)),
                    ));
                }
                _ => {
                    return Err(Error::unknown_field(
                        key,
                        &[field_type::VARIANT, field_type::TUPLE, field_type::STRUCT],
                    ))
                }
            }
        }

        Err(Error::custom("expected enum"))
    }
}

struct ListDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for ListDeserializer<'a> {
    type Value = DynamicList;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListVisitor(self.0))
    }
}

struct ListVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for ListVisitor<'a> {
    type Value = DynamicList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut value = DynamicList::default();

        while let Some(field) = seq.next_element_seed(ReflectDeserializer::new(self.0))? {
            value.push_boxed(field);
        }

        Ok(value)
    }
}

struct SetDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for SetDeserializer<'a> {
    type Value = DynamicSet;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SetVisitor(self.0))
    }
}

struct SetVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for SetVisitor<'a> {
    type Value = DynamicSet;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut value = DynamicSet::default();

        while let Some(field) = seq.next_element_seed(ReflectDeserializer::new(self.0))? {
            value.push_boxed(field);
        }

        Ok(value)
    }
}

struct MapDeserializer<'a>(&'a TypeRegistry);

impl<'a, 'de> DeserializeSeed<'de> for MapDeserializer<'a> {
    type Value = DynamicMap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MapVisitor(self.0))
    }
}

struct MapVisitor<'a>(&'a TypeRegistry);

impl<'a, 'de> Visitor<'de> for MapVisitor<'a> {
    type Value = DynamicMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("dynamic tuple")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut value = DynamicMap::default();

        while let Some((k, v)) = map.next_entry_seed(
            ReflectDeserializer::new(self.0),
            ReflectDeserializer::new(self.0),
        )? {
            value.push_boxed(k, v);
        }

        Ok(value)
    }
}

struct ValueDeserializer<'a>(&'a ReflectDeserialize);

impl<'a, 'de> DeserializeSeed<'de> for ValueDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        self.0.deserialize(&mut erased).map_err(Error::custom)
    }
}
