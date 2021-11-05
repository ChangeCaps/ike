use ike::prelude::*;
use ike_core::AppRunner;
use ike_transform::TransformPlugin;

struct CustomRunner;

impl AppRunner for CustomRunner {
    fn run(&mut self, mut app: App) {
        app.execute_startup();

        loop {
            app.update_components();
            app.execute();

            std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 60.0));
        }
    }
}

fn startup(world: &mut World) {
    let mut parent = world.spawn_node("parent");

    parent.insert(Transform::IDENTITY);

    let parent = parent.entity();

    for x in -20..=20 {
        for z in -20..=20 {
            let mut node = world.spawn_node("node");

            node.insert(Transform::from_xyz(x as f32, 0.0, z as f32));
            node.insert(Parent(parent));
        }
    }
}

fn main() {
    App::new()
        .set_runner(CustomRunner)
        .add_plugin(TransformPlugin)
        .add_exclusive_startup_system(startup)
        .run();
}
