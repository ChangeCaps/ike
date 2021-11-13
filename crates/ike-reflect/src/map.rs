use std::{
    any::{type_name, Any},
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait Map: Reflect {
    fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)>;
    fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)>;
    fn len(&self) -> usize;
    fn clone_dynamic(&self) -> DynamicMap;
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

		impl<K: Reflect, V: Reflect> Map for $name<K, V> {
			#[inline]
			fn get_at(&self, index: usize) -> Option<(&dyn Reflect, &dyn Reflect)> {
				self.iter().nth(index).map(|(k, v)| (k as _, v as _))
			}

			#[inline]
			fn get_at_mut(&mut self, index: usize) -> Option<(&dyn Reflect, &mut dyn Reflect)> {
				self.iter_mut().nth(index).map(|(k, v)| (k as _, v as _))
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

		unsafe impl<K: Reflect, V: Reflect> Reflect for $name<K, V> {
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
		}
	};
}

impl_map!(HashMap, Hash, Eq);
impl_map!(BTreeMap, Ord);
