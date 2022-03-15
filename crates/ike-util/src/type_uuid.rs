use uuid::Uuid;

pub trait TypeUuid {
    const TYPE_UUID: Uuid;
}

pub trait DynTypeUuid {
    fn type_uuid(&self) -> Uuid;
}

impl<T: TypeUuid> DynTypeUuid for T {
    fn type_uuid(&self) -> Uuid {
        Self::TYPE_UUID
    }
}
