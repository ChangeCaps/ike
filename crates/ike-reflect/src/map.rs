use std::{
    any::{type_name, Any},
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;
    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)>;
    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)>;
    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;
    fn insert(&mut self, key: Box<dyn Reflect>, value: Box<dyn Reflect>);
    fn len(&self) -> usize;
    fn clone_dynamic(&self) -> DynamicMap;

    fn keys(&self) -> MapKeysIter<'_>
    where
        Self: Sized,
    {
        MapKeysIter {
            map: self,
            index: 0,
        }
    }
}

pub struct MapKeysIter<'a> {
    map: &'a dyn Map,
    index: usize,
}

impl<'a> Iterator for MapKeysIter<'a> {
    type Item = &'a dyn Reflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.map.get_at(self.index).map(|(_, value)| value);

        self.index += 1;

        value
    }
}

#[derive(Default)]
pub struct DynamicMap {
    name: String,
    values: Vec<(Box<dyn Reflect>, Box<dyn Reflect>)>,
}

impl DynamicMap {
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}

impl Map for DynamicMap {
    #[inline]
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        self.values.iter().find_map(|(k, v)| {
            if k.as_ref() == key {
                Some(v.as_ref())
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
        self.values
            .get(index)
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
    }

    #[inline]
    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
        self.values.get_mut(index).map(|(k, v)| (&**k, v.as_mut()))
    }

    #[inline]
    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let idx = self.values.iter().position(|(k, _)| k.partial_eq(key))?;

        Some(self.values.remove(idx).1)
    }

    #[inline]
    fn insert(&mut self, key: Box<dyn Reflect>, value: Box<dyn Reflect>) {
        self.remove(key.as_ref());

        self.values.push((key, value));
    }

    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn clone_dynamic(&self) -> DynamicMap {
        Self {
            name: self.name.clone(),
            values: self
                .values
                .iter()
                .map(|(k, v)| (k.clone_value(), v.clone_value()))
                .collect(),
        }
    }
}

unsafe impl Reflect for DynamicMap {
    #[inline]
    fn type_name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Map(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Map(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }

    #[inline]
    fn partial_eq(&self, other: &dyn Reflect) -> bool {
        match other.reflect_ref() {
            ReflectRef::Map(other) => {
                if self.len() != other.len() {
                    return false;
                }

                for key in self.keys() {
                    let value = self.get(key).unwrap();

                    if let Some(other_value) = other.get(key) {
                        if value != other_value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }
}

macro_rules! impl_map {
	($name:ident $(, $req:ident)*) => {
		impl<K: FromReflect $(+ $req)*, V: FromReflect> FromReflect for $name<K, V> {
			#[inline]
			fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
				let reflect_map = if let ReflectRef::Map(map) = reflect.reflect_ref() {
					map
				} else {
					return None;
				};

				let mut map = Self::default();

				for i in 0..reflect_map.len() {
					let (k, v) = reflect_map.get_at(i).unwrap();

					let k = FromReflect::from_reflect(k)?;
					let v = FromReflect::from_reflect(v)?;

					map.insert(k, v);
				}

				Some(map)
			}
		}

		impl<K: Reflect + FromReflect $(+ $req)*, V: Reflect + FromReflect> Map for $name<K, V> {
			#[inline]
			fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
				if let Some(key) = key.reflect_into::<K>() {
					self.get(&key).map(|value| value as &dyn Reflect)
				} else {
					None
				}
			}

			#[inline]
			fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
				self.iter().nth(index).map(|(k, v)| (k as _, v as _))
			}

			#[inline]
			fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
				self.iter_mut().nth(index).map(|(k, v)| (k as _, v as _))
			}

			#[inline]
			fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
				if let Some(key) = key.reflect_into::<K>() {
					self.remove(&key).map(|value| Box::new(value) as Box<dyn Reflect>)
				} else {
					None
				}
			} 

			#[inline]
			fn insert(&mut self, key: Box<dyn Reflect>, value: Box<dyn Reflect>) {
				if let Some(key) = key.reflect_into::<K>() {
					if let Some(value) = value.reflect_into::<V>() {
						self.insert(key, value);
					}
				}
			}

			#[inline]
			fn len(&self) -> usize {
				self.len()
			}

			#[inline]
			fn clone_dynamic(&self) -> DynamicMap {
				DynamicMap {
					name: String::from(type_name::<Self>()),
					values: self.iter().map(|(k, v)| (k.clone_value(), v.clone_value())).collect(),
				}
			}
		}

		unsafe impl<K: Reflect + FromReflect $(+ $req)*, V: Reflect + FromReflect> Reflect for $name<K, V> {
			#[inline]
			fn type_name(&self) -> &str {
				type_name::<Self>()
			}

			#[inline]
			fn any(&self) -> &dyn Any {
				self
			}

			#[inline]
			fn any_mut(&mut self) -> &mut dyn Any {
				self
			}

			#[inline]
			fn reflect_ref(&self) -> ReflectRef {
				ReflectRef::Map(self)
			}

			#[inline]
			fn reflect_mut(&mut self) -> ReflectMut {
				ReflectMut::Map(self)
			}

			#[inline]
			fn clone_value(&self) -> Box<dyn Reflect> {
				Box::new(self.clone_dynamic())
			}

			fn partial_eq(&self, other: &dyn Reflect) -> bool {
				match other.reflect_ref() {
					ReflectRef::Map(other) => {
						if self.len() == other.len() {
							return false;
						}

						for key in self.keys() {
							let value = Map::get(self, key).unwrap();

							if let Some(other_value) = other.get(key) {
								if value != other_value {
									return false;
								}
							} else {
								return false;
							}
						}

						false
					}
					_ => false,
				}
			}
		}
	};
}

impl_map!(HashMap, Hash, Eq);
impl_map!(BTreeMap, Ord);
