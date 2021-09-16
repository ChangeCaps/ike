use std::{
    any::{type_name, Any, TypeId},
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::{type_name::TypeName, view::View};

pub trait Drawable {
    type Node;

    fn draw(&self, ctx: &RenderCtx, node: &mut Self::Node);
}

pub struct PassNodeCtx<'a, 'b> {
    pub data: &'a mut PassData,
    pub view: &'a View,
    pub render_ctx: &'a RenderCtx,
    pub render_pass: &'a mut ike_wgpu::RenderPass<'b>,
}

pub trait PassNode<S>: TypeName {
    #[inline]
    fn clear(&mut self) {}

    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, state: &mut S);
}

pub trait RenderPass<S>: TypeName {
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

pub struct RenderCtx {
    pub device: ike_wgpu::Device,
    pub queue: ike_wgpu::Queue,
    pub surface: ike_wgpu::Surface,
    pub config: ike_wgpu::SurfaceConfiguration,
}

pub struct Pass<S: ?Sized> {
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
    pub fn get<P: PassNode<S>>(&self) -> Option<&P> {
        self.nodes.iter().find_map(|(ident, node)| {
            if *ident == type_name::<P>() {
                Some(unsafe { &*(node.as_ref() as *const _ as *const _) })
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn get_mut<P: PassNode<S>>(&mut self) -> Option<&mut P> {
        self.nodes.iter_mut().find_map(|(ident, node)| {
            if *ident == type_name::<P>() {
                Some(unsafe { &mut *(node.as_mut() as *mut _ as *mut _) })
            } else {
                None
            }
        })
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
    pub fn get<N: PassNode<S> + 'static>(&self) -> Option<&N> {
        self.pass.get()
    }

    #[inline]
    pub fn get_mut<N: PassNode<S> + 'static>(&mut self) -> Option<&mut N> {
        self.pass.get_mut()
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

pub struct Renderer<S: ?Sized> {
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
    pub fn frame(&mut self) -> RenderFrame<'_> {
        let passes = self
            .passes
            .iter_mut()
            .map(|(name, pass)| (*name, pass as &mut dyn FramePass))
            .collect();

        RenderFrame { passes }
    }

    #[inline]
    pub fn clear_nodes(&mut self) {
        for pass in self.passes.values_mut() {
            pass.clear();
        }
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
    pub fn push(&mut self, pass: Pass<S>) {
        self.order.push(pass.name());
        self.passes.insert(pass.name(), pass);
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
        let renderer = Self {
            order: Default::default(),
            passes: Default::default(),
        };

        renderer
    }
}

trait FramePass {
    fn clear(&mut self);

    fn node_mut(&mut self, name: &'static str) -> Option<*mut u8>;
}

impl<S> FramePass for Pass<S> {
    #[inline]
    fn clear(&mut self) {
        for (_, node) in &mut self.nodes {
            node.clear();
        }
    }

    #[inline]
    fn node_mut(&mut self, name: &'static str) -> Option<*mut u8> {
        self.nodes.iter_mut().find_map(|(ident, node)| {
            if *ident == name {
                Some(node.as_mut() as *mut _ as *mut _)
            } else {
                None
            }
        })
    }
}

pub struct RenderFrame<'a> {
    passes: BTreeMap<&'a str, &'a mut dyn FramePass>,
}

impl<'a> RenderFrame<'a> {
    #[inline]
    pub fn draw<D: Drawable>(&mut self, ctx: &RenderCtx, drawable: &D) {
        for pass in self.passes.values_mut() {
            if let Some(node) = pass.node_mut(type_name::<D::Node>()) {
                // SAFETY: the implementation on FramePass ensures that the type is always valid
                let node = unsafe { &mut *(node as *mut D::Node) };

                drawable.draw(ctx, node);
            }
        }
    }
}
