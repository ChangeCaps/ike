mod id;
mod label;
mod type_uuid;

use std::{future::Future, pin::Pin};

pub use id::*;
pub use label::*;
pub use type_uuid::*;

pub use anyhow::Error;
pub use ike_macro::uuid;
pub use tracing;
pub use uuid::Uuid;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;
