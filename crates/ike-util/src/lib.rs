mod id;
mod label;

use std::{future::Future, pin::Pin};

pub use id::*;
pub use label::*;

pub use anyhow::Error;
pub use tracing;
pub use uuid::Uuid;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;
