use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    collections::HashMap,
};

#[derive(Default)]
pub struct TypeRegistry {
    registrations: HashMap<TypeId, TypeRegistration>,
    name_to_id: HashMap<Cow<'static, str>, TypeId>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn insert_registration<T: 'static>(&mut self, registration: TypeRegistration) {
        self.registrations.insert(TypeId::of::<T>(), registration);
        self.name_to_id
            .insert(type_name::<T>().into(), TypeId::of::<T>());
    }

    pub fn get_registration<T: 'static>(&self) -> Option<&TypeRegistration> {
        self.registrations.get(&TypeId::of::<T>())
    }

    pub fn get_registration_mut<T: 'static>(&mut self) -> Option<&mut TypeRegistration> {
        self.registrations.get_mut(&TypeId::of::<T>())
    }
}

pub struct TypeRegistration {
    type_id: TypeId,
    data: HashMap<TypeId, Box<dyn TypeData>>,
    name_to_id: HashMap<Cow<'static, str>, TypeId>,
}

impl TypeRegistration {
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn insert<T: TypeData>(&mut self, data: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
        self.name_to_id
            .insert(type_name::<T>().into(), TypeId::of::<T>());
    }

    pub fn data<T: TypeData>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn data_mut<T: TypeData>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }
}

pub trait TypeData: Any + Send + Sync {
    fn clone_type_data(&self) -> Box<dyn TypeData>;
}

impl<T: Clone + Send + Sync + 'static> TypeData for T {
    fn clone_type_data(&self) -> Box<dyn TypeData> {
        Box::new(self.clone())
    }
}

impl dyn TypeData {
    pub fn downcast_ref<T: TypeData>(&self) -> Option<&T> {
        if (*self).type_id() == TypeId::of::<T>() {
            let data = unsafe { &*(self as *const _ as *const T) };

            Some(data)
        } else {
            None
        }
    }

    pub fn downcast_mut<T: TypeData>(&mut self) -> Option<&mut T> {
        if (*self).type_id() == TypeId::of::<T>() {
            let data = unsafe { &mut *(self as *mut _ as *mut T) };

            Some(data)
        } else {
            None
        }
    }
}
