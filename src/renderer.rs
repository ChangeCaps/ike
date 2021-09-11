use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

use glam::Mat4;

use crate::{prelude::Color, type_name::TypeName, view::View};

pub mod render_stage {
    pub const PRE_RENDER: &str = "pre_render";
    pub const RENDER: &str = "render";
}

pub struct PassNodeCtx<'a, 'b> {
    pub data: &'a mut PassData,
    pub view: &'a View,
    pub render_ctx: &'a RenderCtx,
    pub render_pass: &'a mut ike_wgpu::RenderPass<'b>,
}

pub trait PassNode<S>: TypeName {
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, state: &mut S);
}

pub trait RenderPass<S> {
    fn run<'a>(
        &'a mut self,
        encoder: &'a mut ike_wgpu::CommandEncoder,
        ctx: &RenderCtx,
        view: &'a View,
        data: &mut PassData,
        state: &mut S,
    ) -> ike_wgpu::RenderPass<'a>;
}

#[derive(Default)]
pub struct PassData {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl PassData {
    #[inline]
    pub fn insert<T: Any>(&mut self, data: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
    }

    #[inline]
    pub fn register<T: Any + Default>(&mut self) {
        if !self.contains::<T>() {
            self.insert(T::default());
        }
    }

    #[inline]
    pub fn contains<T: Any>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())?.as_ref().downcast_ref()
    }

    #[inline]
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())?
            .as_mut()
            .downcast_mut()
    }
}

#[derive(Clone)]
pub struct SampleCount(pub u32);

impl Default for SampleCount {
    #[inline]
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Clone)]
pub struct TargetFormat(pub ike_wgpu::TextureFormat);

#[derive(Clone)]
pub struct TargetSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct ViewProj(pub Mat4);

#[derive(Default)]
pub struct MainPass {
    pub clear_color: Color,
    pub sample_count: u32,
    width: u32,
    height: u32,
    depth_texture: Option<ike_wgpu::TextureView>,
    ms_texture: Option<ike_wgpu::TextureView>,
}

impl<S> RenderPass<S> for MainPass {
    fn run<'a>(
        &'a mut self,
        encoder: &'a mut ike_wgpu::CommandEncoder,
        ctx: &RenderCtx,
        view: &'a View,
        data: &mut PassData,
        _state: &mut S,
    ) -> ike_wgpu::RenderPass<'a> {
        data.insert(SampleCount(self.sample_count));
        data.insert(TargetFormat(view.format));
        data.insert(TargetSize {
            width: view.width,
            height: view.height,
        });
        data.insert(ViewProj(view.view_proj));

        let depth = if let Some(ref mut depth) = self.depth_texture {
            if self.width != view.width || self.height != view.height {
                let texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                    label: None,
                    size: ike_wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: ike_wgpu::TextureDimension::D2,
                    format: ike_wgpu::TextureFormat::Depth24Plus,
                    usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                if self.sample_count > 1 {
                    let ms_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                        label: None,
                        size: ike_wgpu::Extent3d {
                            width: view.width,
                            height: view.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: self.sample_count,
                        dimension: ike_wgpu::TextureDimension::D2,
                        format: view.format,
                        usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                    });

                    self.ms_texture = Some(ms_texture.create_view(&Default::default()));
                }

                self.width = view.width;
                self.height = view.height;

                let view = texture.create_view(&Default::default());
                *depth = view;
            }

            depth
        } else {
            let depth_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                label: None,
                size: ike_wgpu::Extent3d {
                    width: view.width,
                    height: view.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: ike_wgpu::TextureDimension::D2,
                format: ike_wgpu::TextureFormat::Depth24Plus,
                usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            if self.sample_count > 1 {
                let ms_texture = ctx.device.create_texture(&ike_wgpu::TextureDescriptor {
                    label: None,
                    size: ike_wgpu::Extent3d {
                        width: view.width,
                        height: view.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: ike_wgpu::TextureDimension::D2,
                    format: view.format,
                    usage: ike_wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

                self.ms_texture = Some(ms_texture.create_view(&Default::default()));
            }

            self.width = view.width;
            self.height = view.height;

            self.depth_texture = Some(depth_texture.create_view(&Default::default()));

            self.depth_texture.as_ref().unwrap()
        };

        let color_attachment = if self.sample_count > 1 {
            ike_wgpu::RenderPassColorAttachment {
                view: self.ms_texture.as_ref().unwrap(),
                resolve_target: Some(&view.target),
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        } else {
            ike_wgpu::RenderPassColorAttachment {
                view: &view.target,
                resolve_target: None,
                ops: ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(self.clear_color.into()),
                    store: true,
                },
            }
        };

        let desc = ike_wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(ike_wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(ike_wgpu::Operations {
                    load: ike_wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        };

        encoder.begin_render_pass(&desc)
    }
}

pub struct RenderCtx {
    pub device: ike_wgpu::Device,
    pub queue: ike_wgpu::Queue,
    pub surface: ike_wgpu::Surface,
    pub config: ike_wgpu::SurfaceConfiguration,
}

pub struct Pass<S> {
    data: PassData,
    pass: Box<dyn RenderPass<S>>,
    nodes: Vec<(&'static str, Box<dyn PassNode<S>>)>,
}

impl<S> Pass<S> {
    #[inline]
    pub fn new<P: RenderPass<S> + 'static>(pass: P) -> Self {
        Self {
            data: PassData::default(),
            pass: Box::new(pass),
            nodes: Vec::new(),
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.pass.as_ref().type_name()
    }

    #[inline]
    pub fn push<N: PassNode<S> + 'static>(&mut self, pass: N) {
        self.nodes.push((type_name::<N>(), Box::new(pass)));
    }

    #[inline]
    pub fn before<N: PassNode<S> + 'static, A: PassNode<S> + 'static>(&mut self, pass: N) {
        if let Some(idx) = self
            .nodes
            .iter()
            .position(|(name, _)| *name == type_name::<A>())
        {
            self.nodes.insert(idx, (type_name::<N>(), Box::new(pass)));
        }
    }

    #[inline]
    pub fn after<N: PassNode<S> + 'static, A: PassNode<S> + 'static>(&mut self, pass: N) {
        if let Some(idx) = self
            .nodes
            .iter()
            .position(|(name, _)| *name == type_name::<A>())
        {
            self.nodes
                .insert(idx + 1, (type_name::<N>(), Box::new(pass)));
        }
    }

    #[inline]
    pub fn run(
        &mut self,
        encoder: &mut ike_wgpu::CommandEncoder,
        render_ctx: &RenderCtx,
        view: &View,
        state: &mut S,
    ) {
        let mut render_pass = self
            .pass
            .run(encoder, render_ctx, view, &mut self.data, state);

        let mut ctx = PassNodeCtx {
            data: &mut self.data,
            view,
            render_ctx,
            render_pass: &mut render_pass,
        };

        for (_name, node) in &mut self.nodes {
            node.run(&mut ctx, state);
        }
    }
}

pub struct PassGuard<'a, S, P> {
    pass: &'a mut Pass<S>,
    marker: PhantomData<fn() -> P>,
}

impl<'a, S, P> PassGuard<'a, S, P> {
    #[inline]
    pub fn push<N: PassNode<S> + 'static>(&mut self, pass: N) {
        self.pass.push(pass);
    }

    #[inline]
    pub fn before<N: PassNode<S> + 'static, A: PassNode<S> + 'static>(&mut self, pass: N) {
        self.pass.before::<N, A>(pass);
    }

    #[inline]
    pub fn after<N: PassNode<S> + 'static, A: PassNode<S> + 'static>(&mut self, pass: N) {
        self.pass.after::<N, A>(pass);
    }
}

impl<'a, S, P> std::ops::Deref for PassGuard<'a, S, P> {
    type Target = P;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: is only created by renderer and the inner pass is always P
        unsafe { &*(self.pass.pass.as_ref() as *const _ as *const P) }
    }
}

impl<'a, S, P> std::ops::DerefMut for PassGuard<'a, S, P> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: is only created by renderer and the inner pass is always P
        unsafe { &mut *(self.pass.pass.as_mut() as *mut _ as *mut P) }
    }
}

pub struct Renderer<S> {
    order: Vec<&'static str>,
    passes: HashMap<&'static str, Pass<S>>,
}

impl<S> Renderer<S> {
    #[inline]
    pub fn render_view(&mut self, render_ctx: &RenderCtx, view: &View, state: &mut S) {
        let mut encoder = render_ctx
            .device
            .create_command_encoder(&Default::default());

        for pass in &self.order {
            self.passes
                .get_mut(pass)
                .unwrap()
                .run(&mut encoder, render_ctx, view, state);
        }

        render_ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    #[inline]
    pub fn insert_before<Before: RenderPass<S>>(&mut self, pass: Pass<S>) {
        let idx = self
            .order
            .iter()
            .position(|name| *name == type_name::<Before>())
            .unwrap_or(0);

        self.order.insert(idx, pass.name());
        self.passes.insert(pass.name(), pass);
    }

    #[inline]
    pub fn push_pass<P: RenderPass<S> + 'static>(&mut self, pass: P) {
        self.order.push(type_name::<P>());
        self.passes.insert(type_name::<P>(), Pass::new(pass));
    }

    #[inline]
    pub fn pass_mut<P: RenderPass<S>>(&mut self) -> Option<PassGuard<S, P>> {
        if let Some(pass) = self.passes.get_mut(type_name::<P>()) {
            Some(PassGuard {
                pass,
                marker: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<S> Default for Renderer<S> {
    #[inline]
    fn default() -> Self {
        let mut renderer = Self {
            order: Default::default(),
            passes: Default::default(),
        };

        renderer.push_pass(MainPass::default());

        renderer
    }
}
