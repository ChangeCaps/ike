use ike_core::ResMut;

use crate::{Asset, Assets};

pub fn asset_system<T: Asset>(mut assets: ResMut<Assets<T>>) {
    assets.clean();
}
