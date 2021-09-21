use std::{
    any::{type_name, Any, TypeId},
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::Arc,
};

use glam::UVec2;

use crate::{type_name::TypeName, view::View};

pub trait Drawable {
    type Node: for<'a> FramePassFetch<'a>;

    fn draw(&self, ctx: &RenderCtx, node: <Self::Node as FramePassFetch<'_>>::Item);
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

pub struct RenderNodeCtx<'a> {
    pub data: &'a mut PassData,
    pub view: &'a View,
    pub render_ctx: &'a RenderCtx,
    pub encoder: &'a mut ike_wgpu::CommandEncoder,
}

pub trait RenderNode<S>: TypeName {
    #[inline]
    fn clear(&mut self) {}

    fn run(&mut self, ctx: &mut RenderNodeCtx);
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
pub struct TargetSize(pub UVec2);

pub struct RenderCtx {
    pub device: Arc<ike_wgpu::Device>,
    pub queue: Arc<ike_wgpu::Queue>,
    pub surface: ike_wgpu::Surface,
    pub config: ike_wgpu::SurfaceConfiguration,
}

pub struct Pass<S: ?Sized> {
    pass: Box<dyn RenderPass<S>>,
    nodes: Vec<(&'static str, Box<dyn PassNode<S>>)>,
}

impl<S> Pass<S> {
    #[inline]
    pub fn new<P: RenderPass<S> + 'static>(pass: P) -> Self {
        Self {
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
        data: &mut PassData,
        state: &mut S,
    ) {
        let mut render_pass = self.pass.run(encoder, render_ctx, view, data, state);

        let mut ctx = PassNodeCtx {
            data,
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

enum Node<S: ?Sized> {
    Pass(Pass<S>),
    Node(Box<dyn RenderNode<S>>),
}

pub struct Renderer<S: ?Sized> {
    pass_data: PassData,
    order: Vec<&'static str>,
    nodes: HashMap<&'static str, Node<S>>,
}

impl<S> Renderer<S> {
    #[inline]
    #[cfg(feature = "3d")]
    pub fn default_pbr_pipeline(&mut self) {
        use crate::prelude::*;

        let mut main_pass = MainPass::default();
        main_pass.sample_count = 4;
        main_pass.clear_color = Color::BLUE;

        let mut main_pass = Pass::new(main_pass);

        main_pass.push(HdrCombineNode::default());
        main_pass.push(SpriteNode2d::new());

        let mut hdr_pass = Pass::new(HdrPass::default());

        hdr_pass.push(SkyNode::default());
        hdr_pass.push(DebugNode::default());
        hdr_pass.push(D3Node::default());

        self.push_pass(hdr_pass);
        self.push_node(BloomNode::default());
        self.push_node(AvgLuminanceNode::default());
        self.push_pass(main_pass);
    }

    #[inline]
    pub fn render_view(&mut self, render_ctx: &RenderCtx, view: &View, state: &mut S) {
        let mut encoder = render_ctx
            .device
            .create_command_encoder(&Default::default());

        for pass in &self.order {
            let node = self.nodes.get_mut(pass).unwrap();

            match node {
                Node::Pass(pass) => {
                    pass.run(&mut encoder, render_ctx, view, &mut self.pass_data, state);
                }
                Node::Node(node) => {
                    let mut ctx = RenderNodeCtx {
                        data: &mut self.pass_data,
                        view,
                        render_ctx,
                        encoder: &mut encoder,
                    };

                    node.run(&mut ctx);
                }
            }
        }

        render_ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    #[inline]
    pub fn frame(&mut self) -> RenderFrame<'_> {
        let passes = self
            .nodes
            .iter_mut()
            .filter_map(|(name, node)| match node {
                Node::Pass(pass) => Some((*name, pass as &mut dyn FramePass)),
                Node::Node(_) => None,
            })
            .collect();

        RenderFrame { passes }
    }

    #[inline]
    pub fn clear_nodes(&mut self) {
        for node in self.nodes.values_mut() {
            match node {
                Node::Pass(pass) => pass.clear(),
                Node::Node(node) => node.clear(),
            }
        }
    }

    #[inline]
    pub fn pass_before<Before: RenderPass<S>>(&mut self, pass: Pass<S>) {
        let idx = self
            .order
            .iter()
            .position(|name| *name == type_name::<Before>())
            .unwrap_or(0);

        self.order.insert(idx, pass.name());
        self.nodes.insert(pass.name(), Node::Pass(pass));
    }

    #[inline]
    pub fn push_pass(&mut self, pass: Pass<S>) {
        self.order.push(pass.name());
        self.nodes.insert(pass.name(), Node::Pass(pass));
    }

    #[inline]
    pub fn push_node(&mut self, node: impl RenderNode<S> + 'static) {
        self.order.push(node.type_name());
        self.nodes
            .insert(node.type_name(), Node::Node(Box::new(node)));
    }

    #[inline]
    pub fn pass_mut<P: RenderPass<S>>(&mut self) -> Option<PassGuard<S, P>> {
        if let Some(node) = self.nodes.get_mut(type_name::<P>()) {
            match node {
                Node::Pass(pass) => Some(PassGuard {
                    pass,
                    marker: PhantomData,
                }),
                Node::Node(_) => None,
            }
        } else {
            None
        }
    }
}

impl<S> Default for Renderer<S> {
    #[inline]
    fn default() -> Self {
        let renderer = Self {
            pass_data: Default::default(),
            order: Default::default(),
            nodes: Default::default(),
        };

        renderer
    }
}

pub trait FramePass {
    fn clear(&mut self);

    unsafe fn node_mut(&self, name: &'static str) -> Option<*mut u8>;
}

impl<S> FramePass for Pass<S> {
    #[inline]
    fn clear(&mut self) {
        for (_, node) in &mut self.nodes {
            node.clear();
        }
    }

    #[inline]
    unsafe fn node_mut(&self, name: &'static str) -> Option<*mut u8> {
        self.nodes.iter().find_map(|(ident, node)| {
            if *ident == name {
                Some(node.as_ref() as *const _ as *mut _)
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
            if let Some(item) = unsafe { <D::Node as FramePassFetch<'_>>::fetch(*pass) } {
                drawable.draw(ctx, item);
            }
        }
    }
}

pub unsafe trait FramePassFetch<'a> {
    type Item;

    fn fetch_ids() -> Vec<TypeId>;

    unsafe fn fetch(pass: &dyn FramePass) -> Option<Self::Item>;
}

unsafe impl<'a, P: 'static> FramePassFetch<'a> for &mut P {
    type Item = &'a mut P;

    #[inline]
    fn fetch_ids() -> Vec<TypeId> {
        vec![TypeId::of::<P>()]
    }

    #[inline]
    unsafe fn fetch(pass: &dyn FramePass) -> Option<Self::Item> {
        let node = unsafe { pass.node_mut(type_name::<P>())? };

        unsafe { Some(&mut *(node as *mut P)) }
    }
}

unsafe impl<'a, P: 'static> FramePassFetch<'a> for Option<&mut P> {
    type Item = Option<&'a mut P>;

    #[inline]
    fn fetch_ids() -> Vec<TypeId> {
        vec![TypeId::of::<P>()]
    }

    #[inline]
    unsafe fn fetch(pass: &dyn FramePass) -> Option<Self::Item> {
        Some(unsafe {
            pass.node_mut(type_name::<P>())
                .map(|node| &mut *(node as *mut P))
        })
    }
}

macro_rules! check {
    ($to_check:ident | $($ident:ident),*) => {
        check!($to_check: $($ident),*);
    };
    ($to_check:ident $(,)? $($rest:ident),+ | $($ident:ident),*) => {
        check!($to_check: $($ident),*);

        check!($($rest),* | $($ident),*);
    };
    ($to_check:ident: $($ident:ident),*) => {
        for id in $to_check::fetch_ids() {
            if $(($ident::fetch_ids().contains(&id) && stringify!($to_check) != stringify!($ident)))||* {
                panic!("invalid FramePassFetch, '{}' fetched twice", type_name::<$to_check>());
            }
        }
    }
}

macro_rules! impl_pass_fetch {
    ($($ident:ident),*) => {
        unsafe impl<'a, $($ident: FramePassFetch<'a> + 'static),*> FramePassFetch<'a> for ($($ident,)*) {
            type Item = ($(<$ident as FramePassFetch<'a>>::Item,)*);

            fn fetch_ids() -> Vec<TypeId> {
                let mut v = Vec::new();

                $(
                    v.append(&mut $ident::fetch_ids());
                )*

                return v;
            }

            #[inline]
            unsafe fn fetch(pass: &dyn FramePass) -> Option<Self::Item> {
                check!($($ident),* | $($ident),*);

                Some(
                    ($(unsafe { $ident::fetch(pass)? },)*)
                )
            }
        }
    };
}

impl_pass_fetch!(A);
impl_pass_fetch!(A, B);
impl_pass_fetch!(A, B, C);
impl_pass_fetch!(A, B, C, D);
impl_pass_fetch!(A, B, C, D, E);
impl_pass_fetch!(A, B, C, D, E, F);
impl_pass_fetch!(A, B, C, D, E, F, G);
impl_pass_fetch!(A, B, C, D, E, F, G, H);
impl_pass_fetch!(A, B, C, D, E, F, G, H, I);
impl_pass_fetch!(A, B, C, D, E, F, G, H, I, J);
impl_pass_fetch!(A, B, C, D, E, F, G, H, I, J, K);
impl_pass_fetch!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_pass_fetch!(A, B, C, D, E, F, G, H, I, J, K, L, M);
