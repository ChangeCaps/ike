use std::{any::type_name, collections::HashMap};

use egui::CtxRef;

use crate::{
    renderer::{RenderCtx, Renderer},
    view::Views,
};

pub struct UiPanels<S: 'static> {
    order: Vec<&'static str>,
    panels: HashMap<&'static str, Box<dyn UiPanel<S>>>,
}

impl<S> UiPanels<S> {
    #[inline]
    pub fn insert<T: UiPanel<S>>(&mut self, panel: T) {
        self.order.push(type_name::<T>());
        self.panels.insert(type_name::<T>(), Box::new(panel));
    }

    #[inline]
    pub fn insert_after<T: UiPanel<S>, After: UiPanel<S>>(&mut self, panel: T) {
        let idx = self
            .order
            .iter()
            .position(|name| *name == type_name::<After>())
            .unwrap();

        self.order.insert(idx + 1, type_name::<T>());
        self.panels.insert(type_name::<T>(), Box::new(panel));
    }

    #[inline]
    pub fn get<T: UiPanel<S>>(&self) -> Option<&T> {
        // Safety: since all insertion is type checked, casting when getting is safe.
        unsafe { Some(&*(self.panels.get(type_name::<T>())?.as_ref() as *const _ as *const T)) }
    }

    #[inline]
    pub fn get_mut<T: UiPanel<S>>(&mut self) -> Option<&mut T> {
        // Safety: since all insertion is type checked, casting when getting is safe.
        unsafe { Some(&mut *(self.panels.get_mut(type_name::<T>())?.as_mut() as *mut _ as *mut T)) }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn UiPanel<S>>> {
        self.panels.values()
    }

    #[inline]
    pub fn show(&mut self, ctx: &mut UiPanelCtx<S>) {
        for panel in &self.order {
            self.panels.get_mut(panel).unwrap().show(ctx);
        }
    }
}

impl<S> Default for UiPanels<S> {
    #[inline]
    fn default() -> Self {
        Self {
            order: Vec::new(),
            panels: HashMap::new(),
        }
    }
}

pub struct UiPanelCtx<'a, S> {
    pub egui_ctx: &'a CtxRef,
    pub render_ctx: &'a RenderCtx,
    pub renderer: &'a mut Renderer<S>,
    pub views: &'a Views,
    pub state: &'a mut S,
}

pub trait UiPanel<S>: 'static {
    fn show(&mut self, ctx: &mut UiPanelCtx<S>);
}
