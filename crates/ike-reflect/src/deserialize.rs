use ike_type::TypeRegistry;

pub struct ReflectDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}
