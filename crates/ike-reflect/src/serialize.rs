use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

use crate::{type_field, Map, Reflect, ReflectRef, Struct, TupleStruct};

impl Serialize for dyn Reflect {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.reflect_ref() {
            ReflectRef::Struct(value) => {
                let mut state = serializer.serialize_map(Some(2))?;

                state.serialize_entry(type_field::TYPE, value.type_name())?;
                state.serialize_entry(type_field::STRUCT, value)?;

                state.end()
            }
            ReflectRef::TupleStruct(value) => {
                let mut state = serializer.serialize_map(Some(2))?;

                state.serialize_entry(type_field::TYPE, value.type_name())?;
                state.serialize_entry(type_field::TUPLE_STRUCT, value)?;

                state.end()
            }
            ReflectRef::Value(value) => {
                let mut state = serializer.serialize_map(Some(2))?;

                state.serialize_entry(type_field::TYPE, value.type_name())?;
                state.serialize_entry(type_field::VALUE, value.serialize())?;

                state.end()
            }
            _ => unimplemented!(),
        }
    }
}

impl<'a> Serialize for dyn Struct {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.field_len()))?;

        for (i, value) in self.iter_fields().enumerate() {
            let name = self.name_at(i).unwrap();
            state.serialize_entry(name, value)?;
        }

        state.end()
    }
}

impl<'a> Serialize for dyn TupleStruct {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.field_len()))?;

        for i in 0..self.field_len() {
            let value = self.field(i).unwrap();
            state.serialize_element(value)?;
        }

        state.end()
    }
}

impl<'a> Serialize for dyn Map {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.len()))?;

        for i in 0..self.len() {
            let (k, v) = self.get_at(i).unwrap();

            state.serialize_entry(k, v)?;
        }

        state.end()
    }
}
