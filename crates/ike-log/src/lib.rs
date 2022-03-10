use ike_app::{App, Plugin};
use ike_util::tracing::subscriber;
use tracing_subscriber::prelude::*;

#[derive(Default)]
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(self, _app: &mut App) {
        let subscriber = tracing_subscriber::registry();

        #[cfg(feature = "tracing-tracy")]
        let subscriber = subscriber.with(tracing_tracy::TracyLayer::new());

        subscriber::set_global_default(subscriber)
            .expect("failed to set up global default subscriber");
    }
}
