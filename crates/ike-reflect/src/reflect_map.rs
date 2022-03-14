use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectMap: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;
    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)>;
    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)>;
    fn len(&self) -> usize;
    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;
    fn insert(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Result<(), (Box<dyn Reflect>, Box<dyn Reflect>)>;

    fn partial_eq(&self, other: &dyn ReflectMap) -> bool {
        if self.type_name() != other.type_name() || self.len() != other.len() {
            return false;
        }

        for index in 0..self.len() {
            let (key, value) = self.get_at(index).unwrap();

            if let Some(other_value) = other.get(key) {
                if !value.partial_eq(other_value) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

struct DynamicEntry {
    key: Box<dyn Reflect>,
    value: Box<dyn Reflect>,
}

#[derive(Default)]
pub struct DynamicMap {
    name: String,
    entries: Vec<DynamicEntry>,
}

impl DynamicMap {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn push_boxed(&mut self, key: Box<dyn Reflect>, value: Box<dyn Reflect>) {
        self.entries.push(DynamicEntry { key, value });
    }
}

impl ReflectMap for DynamicMap {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.key.partial_eq(key))?;

        Some(self.entries[index].value.as_ref())
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.entries
            .get(index)
            .map(|entry| (entry.key.as_ref(), entry.value.as_ref()))
    }

    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
        self.entries
            .get_mut(index)
            .map(|entry| (entry.key.as_ref(), entry.value.as_mut()))
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.key.partial_eq(key))?;

        Some(self.entries.remove(index).value)
    }

    fn insert(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Result<(), (Box<dyn Reflect>, Box<dyn Reflect>)> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.key.partial_eq(key.as_ref()));

        if index.is_some() {
            Err((key, value))
        } else {
            self.push_boxed(key, value);

            Ok(())
        }
    }
}

impl Reflect for DynamicMap {
    fn type_name(&self) -> &str {
        &self.name
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }
}

impl<K: Reflect + Eq + Hash, V: Reflect> Reflect for HashMap<K, V> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }
}

impl<K: Reflect + Eq + Hash, V: Reflect> ReflectMap for HashMap<K, V> {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        let key = key.downcast_ref()?;

        self.get(key).map(|value| value as _)
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.iter().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
        self.iter_mut().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let key = key.downcast_ref::<K>()?;

        Some(Box::new(self.remove(key)?))
    }

    fn insert(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Result<(), (Box<dyn Reflect>, Box<dyn Reflect>)> {
        if key.as_ref().is::<K>() && value.as_ref().is::<V>() {
            if let (Ok(key), Ok(value)) = (key.downcast(), value.downcast()) {
                self.insert(*key, *value);

                Ok(())
            } else {
                unreachable!()
            }
        } else {
            Err((key, value))
        }
    }
}

impl<K: FromReflect + Eq + Hash, V: FromReflect> FromReflect for HashMap<K, V> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let map = reflect.reflect_ref().get_map()?;

        let mut this = HashMap::new();

        for i in 0..map.len() {
            let (k, v) = map.get_at(i)?;
            this.insert(K::from_reflect(k)?, V::from_reflect(v)?);
        }

        Some(this)
    }
}

impl<K: Reflect + Ord, V: Reflect> Reflect for BTreeMap<K, V> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }
}

impl<K: Reflect + Ord, V: Reflect> ReflectMap for BTreeMap<K, V> {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        let key = key.downcast_ref()?;

        self.get(key).map(|value| value as _)
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.iter().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
        self.iter_mut().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let key = key.downcast_ref()?;

        Some(Box::new(self.remove(key)?))
    }

    fn insert(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Result<(), (Box<dyn Reflect>, Box<dyn Reflect>)> {
        if key.as_ref().is::<K>() && value.as_ref().is::<V>() {
            if let (Ok(key), Ok(value)) = (key.downcast(), value.downcast()) {
                self.insert(*key, *value);

                Ok(())
            } else {
                unreachable!()
            }
        } else {
            Err((key, value))
        }
    }
}

impl<K: FromReflect + Ord, V: FromReflect> FromReflect for BTreeMap<K, V> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let map = reflect.reflect_ref().get_map()?;

        let mut this = BTreeMap::new();

        for i in 0..map.len() {
            let (k, v) = map.get_at(i)?;
            this.insert(K::from_reflect(k)?, V::from_reflect(v)?);
        }

        Some(this)
    }
}
