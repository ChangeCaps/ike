use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectList: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn len(&self) -> usize;

    fn partial_eq(&self, other: &dyn ReflectList) -> bool {
        if self.type_name() != other.type_name() || self.len() != other.len() {
            return false;
        }

        for index in 0..self.len() {
            if let (Some(a), Some(b)) = (self.get(index), other.get(index)) {
                if !a.partial_eq(b) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

#[derive(Default)]
pub struct DynamicList {
    name: String,
    entries: Vec<Box<dyn Reflect>>,
}

impl DynamicList {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn push_boxed(&mut self, entry: Box<dyn Reflect>) {
        self.entries.push(entry);
    }
}

impl ReflectList for DynamicList {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.entries.get(index).map(|entry| entry.as_ref())
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.entries.get_mut(index).map(|entry| entry.as_mut())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Reflect for DynamicList {
    fn type_name(&self) -> &str {
        &self.name
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::List(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::List(self)
    }
}

impl<T: Reflect> Reflect for Vec<T> {
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::List(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::List(self)
    }
}

impl<T: Reflect> ReflectList for Vec<T> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        if index < self.len() {
            Some(&mut self[index])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: FromReflect> FromReflect for Vec<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let list = reflect.reflect_ref().get_list()?;

        let mut this = Vec::new();

        for index in 0..list.len() {
            let element = list.get(index)?;
            this.push(T::from_reflect(element)?);
        }

        Some(this)
    }
}
