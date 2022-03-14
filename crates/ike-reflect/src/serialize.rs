use crate::{
    field_type, Reflect, ReflectEnum, ReflectList, ReflectMap, ReflectRef, ReflectSet,
    ReflectStruct, ReflectTuple, VariantRef,
};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

impl Serialize for dyn Reflect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        match self.reflect_ref() {
            ReflectRef::Tuple(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::TUPLE, reflect)?;
            }
            ReflectRef::Struct(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::STRUCT, reflect)?;
            }
            ReflectRef::Enum(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::ENUM, reflect)?;
            }
            ReflectRef::List(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::LIST, reflect)?;
            }
            ReflectRef::Set(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::SET, reflect)?;
            }
            ReflectRef::Map(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::MAP, reflect)?;
            }
            ReflectRef::Value(reflect) => {
                state.serialize_entry(field_type::TYPE, reflect.type_name())?;
                state.serialize_entry(field_type::VALUE, reflect.serialize())?;
            }
        }

        state.end()
    }
}

impl Serialize for dyn ReflectTuple {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.field_len()))?;

        for i in 0..self.field_len() {
            state.serialize_element(self.field(i).unwrap())?;
        }

        state.end()
    }
}

impl Serialize for dyn ReflectStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.field_len()))?;

        for i in 0..self.field_len() {
            let name = self.name_at(i).unwrap();
            state.serialize_entry(name, self.field_at(i).unwrap())?;
        }

        state.end()
    }
}

impl Serialize for dyn ReflectEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        state.serialize_entry(field_type::VARIANT, self.variant_name())?;

        match self.variant_ref() {
            VariantRef::Tuple(reflect) => {
                state.serialize_entry(field_type::TUPLE, reflect)?;
            }
            VariantRef::Struct(reflect) => {
                state.serialize_entry(field_type::STRUCT, reflect)?;
            }
        }

        state.end()
    }
}

impl Serialize for dyn ReflectList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.len()))?;

        for i in 0..self.len() {
            state.serialize_element(self.get(i).unwrap())?;
        }

        state.end()
    }
}

impl Serialize for dyn ReflectSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.len()))?;

        for i in 0..self.len() {
            state.serialize_element(self.get_at(i).unwrap())?;
        }

        state.end()
    }
}

impl Serialize for dyn ReflectMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.len()))?;

        for i in 0..self.len() {
            let (key, value) = self.get_at(i).unwrap();
            state.serialize_entry(key, value)?;
        }

        state.end()
    }
}
