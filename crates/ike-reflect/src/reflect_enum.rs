use crate::Reflect;

pub trait ReflectEnum: Reflect {
    fn variant_name(&self) -> &str;
    fn variant(&self) -> &dyn Reflect;
    fn variant_mut(&mut self) -> &mut dyn Reflect;
}
