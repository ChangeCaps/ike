use std::{any::type_name, collections::HashMap};

use crate::view::View;

pub mod render_stage {
    pub const PRE_RENDER: &str = "pre_render";
    pub const RENDER: &str = "render";
}

pub trait RenderNode<S> {
    fn run(&mut self, ctx: &RenderCtx, view: &View, state: &mut S);
}

pub struct RenderCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct Renderer<S> {
    order: Vec<&'static str>,
    stages: HashMap<&'static str, Vec<&'static str>>,
    nodes: HashMap<&'static str, Box<dyn RenderNode<S>>>,
}

impl<S> Renderer<S> {
    pub fn render_view(&mut self, render_ctx: &RenderCtx, view: &View, state: &mut S) {
        for stage in &self.order {
            for node in self.stages.get(stage).unwrap() {
                self.nodes
                    .get_mut(node)
                    .unwrap()
                    .run(render_ctx, view, state);
            }
        }
    }

    #[inline]
    pub fn push_stage(&mut self, stage: &'static str) {
        self.order.push(stage);
        self.stages.insert(stage, Vec::new());
    }

    #[inline]
    pub fn add_node<T: RenderNode<S> + 'static>(&mut self, node: T) {
        self.add_node_to_stage(render_stage::RENDER, node);
    }

    #[inline]
    pub fn add_node_to_stage<T: RenderNode<S> + 'static>(&mut self, stage: &str, node: T) {
        self.stages.get_mut(stage).unwrap().push(type_name::<T>());
        self.nodes.insert(type_name::<T>(), Box::new(node));
    }

    #[inline]
    pub fn get_node<T: RenderNode<S>>(&self) -> Option<&T> {
        unsafe { Some(&*(self.nodes.get(type_name::<T>())? as *const _ as *const T)) }
    }

    #[inline]
    pub fn get_node_mut<T: RenderNode<S>>(&mut self) -> Option<&mut T> {
        unsafe { Some(&mut *(self.nodes.get_mut(type_name::<T>())? as *mut _ as *mut T)) }
    }
}

impl<S> Default for Renderer<S> {
    fn default() -> Self {
        let mut renderer = Self {
            order: Default::default(),
            stages: Default::default(),
            nodes: Default::default(),
        };

        renderer.push_stage(render_stage::PRE_RENDER);
        renderer.push_stage(render_stage::RENDER);

        renderer
    }
}
