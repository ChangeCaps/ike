use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectSet: Reflect {
    fn get(&self, key: &dyn Reflect) -> bool;
    fn get_at(&self, index: usize) -> Option<&dyn Reflect>;
    fn len(&self) -> usize;
    fn remove(&mut self, key: &dyn Reflect) -> bool;
    fn insert(&mut self, key: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>>;
}

impl dyn ReflectSet {
    pub fn partial_eq(&self, other: &dyn ReflectSet) -> bool {
        if self.type_name() != other.type_name() || self.len() != other.len() {
            return false;
        }

        for index in 0..self.len() {
            let key = self.get_at(index).unwrap();

            if !other.get(key) {
                return false;
            }
        }

        true
    }

    pub fn clone_dynamic(&self) -> DynamicSet {
        let mut this = DynamicSet::default();
        this.set_name(self.type_name());

        for index in 0..self.len() {
            let entry = self.get_at(index).unwrap();
            let _ = this.insert(entry.clone_dynamic());
        }

        this
    }
}

#[derive(Default)]
pub struct DynamicSet {
    name: String,
    entries: Vec<Box<dyn Reflect>>,
}

impl DynamicSet {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn push_boxed(&mut self, entry: Box<dyn Reflect>) {
        self.entries.push(entry);
    }
}

impl ReflectSet for DynamicSet {
    fn get(&self, key: &dyn Reflect) -> bool {
        self.entries
            .iter()
            .find(|entry| entry.partial_eq(key))
            .is_some()
    }

    fn get_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.entries.get(index).map(|entry| entry.as_ref())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> bool {
        let index = self.entries.iter().position(|entry| entry.partial_eq(key));

        if let Some(index) = index {
            self.entries.remove(index);

            true
        } else {
            false
        }
    }

    fn insert(&mut self, key: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.partial_eq(key.as_ref()));

        if let Some(index) = index {
            let removed = self.entries.remove(index);

            self.entries.push(key);

            Err(removed)
        } else {
            self.entries.push(key);

            Ok(())
        }
    }
}

impl Reflect for DynamicSet {
    fn type_name(&self) -> &str {
        &self.name
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Set(self)
    }
}

impl<T: Reflect + FromReflect + Eq + Hash> Reflect for HashSet<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Set(self)
    }
}

impl<T: Reflect + FromReflect + Eq + Hash> ReflectSet for HashSet<T> {
    fn get(&self, key: &dyn Reflect) -> bool {
        if let Some(key) = key.downcast_ref() {
            self.contains(key)
        } else {
            false
        }
    }

    fn get_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.iter().nth(index).map(|key| key as _)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> bool {
        if let Some(key) = key.downcast_ref() {
            self.remove(key)
        } else {
            false
        }
    }

    fn insert(&mut self, key: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        self.insert(T::from_reflect(key.as_ref()).ok_or(key)?);

        Ok(())
    }
}

impl<T: FromReflect + Eq + Hash> FromReflect for HashSet<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let set = reflect.reflect_ref().get_set()?;

        let mut this = HashSet::new();

        for index in 0..set.len() {
            let key = set.get_at(index)?;
            this.insert(T::from_reflect(key)?);
        }

        Some(this)
    }
}

impl<T: Reflect + FromReflect + Ord> Reflect for BTreeSet<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Set(self)
    }
}

impl<T: Reflect + FromReflect + Ord> ReflectSet for BTreeSet<T> {
    fn get(&self, key: &dyn Reflect) -> bool {
        if let Some(key) = key.downcast_ref() {
            self.contains(key)
        } else {
            false
        }
    }

    fn get_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.iter().nth(index).map(|key| key as _)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> bool {
        if let Some(key) = key.downcast_ref() {
            self.remove(key)
        } else {
            false
        }
    }

    fn insert(&mut self, key: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        self.insert(T::from_reflect(key.as_ref()).ok_or(key)?);

        Ok(())
    }
}

impl<T: FromReflect + Ord> FromReflect for BTreeSet<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let set = reflect.reflect_ref().get_set()?;

        let mut this = BTreeSet::new();

        for index in 0..set.len() {
            let key = set.get_at(index)?;
            this.insert(T::from_reflect(key)?);
        }

        Some(this)
    }
}
