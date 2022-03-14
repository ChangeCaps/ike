use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectSet: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;
    fn len(&self) -> usize;
    fn remove(&mut self, key: &dyn Reflect) -> bool;
    fn insert(&mut self, key: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>>;
}

impl<T: Reflect + Eq + Hash> Reflect for HashSet<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Set(self)
    }
}

impl<T: Reflect + Eq + Hash> ReflectSet for HashSet<T> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
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
        self.insert(*key.downcast()?);

        Ok(())
    }
}

impl<T: FromReflect + Eq + Hash> FromReflect for HashSet<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let set = reflect.reflect_ref().get_set()?;

        let mut this = HashSet::new();

        for index in 0..set.len() {
            let key = set.get(index)?;
            this.insert(T::from_reflect(key)?);
        }

        Some(this)
    }
}

impl<T: Reflect + Ord> Reflect for BTreeSet<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Set(self)
    }
}

impl<T: Reflect + Ord> ReflectSet for BTreeSet<T> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
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
        self.insert(*key.downcast()?);

        Ok(())
    }
}

impl<T: FromReflect + Ord> FromReflect for BTreeSet<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let set = reflect.reflect_ref().get_set()?;

        let mut this = BTreeSet::new();

        for index in 0..set.len() {
            let key = set.get(index)?;
            this.insert(T::from_reflect(key)?);
        }

        Some(this)
    }
}
