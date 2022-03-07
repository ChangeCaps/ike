use ike_ecs::{EventWriter, ResMut};

use crate::{Asset, Assets, Handle};

#[derive(Debug)]
pub enum AssetEvent<T: Asset> {
    Created { handle: Handle<T> },
    Modified { handle: Handle<T> },
    Removed { handle: Handle<T> },
}

impl<T: Asset> AssetEvent<T> {
    pub fn system(mut event_writer: EventWriter<Self>, mut assets: ResMut<Assets<T>>) {
        for event in assets.events.drain(..) {
            event_writer.send(event);
        }
    }
}
