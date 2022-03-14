use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectMap: Reflect {
    fn get(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)>;
    fn get_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)>;
    fn len(&self) -> usize;
    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;
    fn insert(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Result<(), (Box<dyn Reflect>, Box<dyn Reflect>)>;
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
    fn get(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.iter().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn get_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
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
            let (k, v) = map.get(i)?;
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
    fn get(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.iter().nth(index).map(|(k, v)| (k as _, v as _))
    }

    fn get_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
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
            let (k, v) = map.get(i)?;
            this.insert(K::from_reflect(k)?, V::from_reflect(v)?);
        }

        Some(this)
    }
}
