use std::borrow::Cow;

pub enum ShaderSource<'a> {
    Wgsl(Cow<'a, str>),
}

pub struct ShaderModuleDescriptor<'a> {
    pub label: Option<&'a str>,
    pub source: ShaderSource<'a>,
}

pub(crate) trait ShaderModuleTrait {}

#[cfg(feature = "wgpu")]
impl ShaderModuleTrait for wgpu::ShaderModule {}

pub struct ShaderModule(pub(crate) Box<dyn ShaderModuleTrait>);