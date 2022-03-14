use crate::Reflect;

pub trait ReflectStruct: Reflect {
    fn field(&mut self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn name_at(&self, index: usize) -> Option<&str>;
    fn field_len(&self) -> usize;
}
