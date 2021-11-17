use std::{any::type_name, hash::Hash, marker::PhantomData, path::PathBuf, sync::Arc};

use ike_core::Resources;
use ike_derive::Reflect;
use ike_reflect::{
    egui::{self, popup, ScrollArea},
    Inspect,
};
use serde::{Deserialize, Serialize};

use crate::Assets;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleUntyped {
    Path(PathBuf),
    Id(u64),
}

pub trait IntoHandleUntyped {
    fn into_handle_untyped(self) -> HandleUntyped;
}

impl IntoHandleUntyped for u64 {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Id(self)
    }
}

impl IntoHandleUntyped for &str {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Path(self.into())
    }
}

impl IntoHandleUntyped for PathBuf {
    #[inline]
    fn into_handle_untyped(self) -> HandleUntyped {
        HandleUntyped::Path(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Inner {
    Tracked(Arc<HandleUntyped>),
    Untracked(HandleUntyped),
}

impl Inner {
    #[inline]
    fn untyped(&self) -> &HandleUntyped {
        match self {
            Self::Tracked(handle) => handle,
            Self::Untracked(handle) => handle,
        }
    }
}

pub trait Asset: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Asset for T {}

#[derive(Reflect, Serialize, Deserialize)]
#[serde(bound = "T: 'static")]
#[reflect(value)]
pub struct Handle<T: Asset> {
    inner: Inner,
    marker: PhantomData<&'static T>,
}

unsafe impl<T: Asset> Send for Handle<T> {}
unsafe impl<T: Asset> Sync for Handle<T> {}

impl<T: Asset> Handle<T> {
    #[inline]
    pub fn new<U: IntoHandleUntyped>(untyped: U) -> Self {
        Self {
            inner: Inner::Tracked(Arc::new(untyped.into_handle_untyped())),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn new_untracked<U: IntoHandleUntyped>(untyped: U) -> Self {
        Self {
            inner: Inner::Untracked(untyped.into_handle_untyped()),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn new_rand() -> Self {
        Self::new(rand::random::<u64>())
    }

    #[inline]
    pub fn untyped(&self) -> &HandleUntyped {
        self.inner.untyped()
    }

    #[inline]
    pub fn tracked(&self) -> Option<&Arc<HandleUntyped>> {
        match self.inner {
            Inner::Tracked(ref tracked) => Some(tracked),
            _ => None,
        }
    }

    #[inline]
    pub fn shared(&self) -> bool {
        self.tracked()
            .map(|tracked| Arc::strong_count(tracked) > 1)
            .unwrap_or(true)
    }
}

impl<T: Asset> Default for Handle<T> {
    #[inline]
    fn default() -> Self {
        Self::new_rand()
    }
}

impl<T: Asset> Inspect for Handle<T> {
    #[inline]
    fn inspect(&mut self, ui: &mut egui::Ui, resources: &Resources) -> egui::Response {
        ui.horizontal(|ui| {
            let assets = resources.read::<Assets<T>>().unwrap();

            match self.inner.untyped() {
                HandleUntyped::Id(id) => {
                    ui.label(id);
                }
                HandleUntyped::Path(path) => {
                    ui.label(path.display());
                }
            };

            let popup_id = ui.make_persistent_id(type_name::<Self>());
            let response = ui.button("*");

            if response.clicked() {
                ui.memory().toggle_popup(popup_id);
            }

            popup::popup_below_widget(ui, popup_id, &response, |ui| {
                ui.set_max_width(300.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for handle in assets.handles() {
                        let handle_response = match handle.untyped() {
                            HandleUntyped::Id(id) => ui.button(id),
                            HandleUntyped::Path(path) => ui.button(path.display()),
                        };

                        if handle_response.clicked() {
                            *self = handle.clone();
                        }
                    }
                });
            });

            response
        })
        .inner
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.untyped().eq(other.untyped())
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> Hash for Handle<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.untyped().hash(state)
    }
}

impl<T: Asset> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: Asset> std::fmt::Debug for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
