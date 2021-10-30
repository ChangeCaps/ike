use std::panic::AssertUnwindSafe;

use crate::{
    renderer::{RenderCtx, RenderGraph},
    state::{EditorCtx, StartCtx, State, UpdateCtx},
    view::Views,
};

pub struct App<S: 'static> {
    pub state: S,
    pub render_graph: RenderGraph,
}

impl<S> App<S> {}
