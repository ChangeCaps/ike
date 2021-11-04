use ike_core::ResMut;

use crate::Assets;

pub fn asset_system<T>(mut assets: ResMut<Assets<T>>) {
    assets.clean();
}
