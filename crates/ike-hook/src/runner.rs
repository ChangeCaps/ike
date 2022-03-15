use ike_app::{App, AppRunner};

pub struct HookRunner {
    pub port: u16,
}

impl HookRunner {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl AppRunner for HookRunner {
    fn run(self: Box<Self>, app: App) {}
}
