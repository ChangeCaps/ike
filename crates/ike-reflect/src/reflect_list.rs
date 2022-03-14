use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectList: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn len(&self) -> usize;
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
