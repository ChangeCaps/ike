use ike::prelude::*;
use ike_transform::TransformPlugin;

struct Rotate;

impl Component for Rotate {
    fn update(&mut self, node: &mut Node<'_>, _world: &World) {
        let mut transform = node.get_component_mut::<Transform>().unwrap();

        transform.rotation *= Quat::from_rotation_y(0.01);
    }
}

struct CameraRotate(Vec2);

impl Component for CameraRotate {
    fn update(&mut self, node: &mut Node<'_>, world: &World) {
        let mouse = world.read_resource::<Mouse>().unwrap();

        if mouse.grabbed {
            self.0 += mouse.movement * 0.001 * -1.0;
        }

        let key_input = world.read_resource::<Input<Key>>().unwrap();

        let mut transform = &mut *node.get_component_mut::<Transform>().unwrap();

        transform.rotation = Quat::from_rotation_y(self.0.x);
        transform.rotation *= Quat::from_rotation_x(self.0.y);

        if key_input.down(&Key::W) {
            transform.translation -= transform.local_z() * 0.1;
        }

        if key_input.down(&Key::S) {
            transform.translation += transform.local_z() * 0.1;
        }

        if key_input.down(&Key::D) {
            transform.translation += transform.local_x() * 0.1;
        }

        if key_input.down(&Key::A) {
            transform.translation -= transform.local_x() * 0.1;
        }
    }
}

struct Started;

struct StartupSystem;

impl ExclusiveSystem for StartupSystem {
    fn run(&mut self, world: &mut World) {
        if world.has_resource::<Started>() {
            return;
        }

        world.insert_resource(Started);

        let hdr = HdrTexture::load("assets/env.hdr").unwrap();

        let mut env = Environment::default();
        env.load(&hdr);

        let env = world
            .write_resource::<Assets<Environment>>()
            .unwrap()
            .add(env);
        world.insert_resource(env);

        let mut node = world.spawn_node("Camera");

        let mut transform = Transform::from_xyz(2.0, 3.0, 1.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);

        node.insert(PerspectiveProjection::default());
        node.insert(transform);
        node.insert(CameraRotate(Vec2::ZERO));

        world.write_resource::<MainCamera>().unwrap().0 = Some(node.entity());

        let mut light = world.spawn_node("light");

        light.insert(Transform::from_xyz(0.0, 1.0, 0.0));
        light.insert(DirectionalLight::default());

        let mut meshes = world.write_resource::<Assets<Mesh>>().unwrap();

        let mesh = meshes.add(Mesh::cube(Vec3::ONE / 2.0));

        let mut materials = world.write_resource::<Assets<PbrMaterial>>().unwrap();

        let material = materials.add(PbrMaterial {
            roughness: 0.01,
            metallic: 0.01,
            reflectance: 0.5,
            ..PbrMaterial::default()
        });

        let mut node = world.spawn_node("rotate");

        node.insert(Transform::from_scale(Vec3::new(20.0, 0.5, 20.0)));
        node.insert(mesh.clone());
        node.insert(material.clone());

        let mut cube = world.spawn_node("cube");

        cube.insert(Transform::from_xyz(0.0, 1.0, 0.0));
        cube.insert(mesh.clone());
        cube.insert(material.clone());
    }
}

fn camera_aspect_system(window: Res<Window>, query: Query<&mut PerspectiveProjection>) {
    let aspect = window.aspect();

    for projection in query {
        projection.aspect = aspect;
    }
}

fn window_capture_system(
    mut mouse: ResMut<Mouse>,
    key_input: Res<Input<Key>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if mouse_input.pressed(&MouseButton::Left) {
        mouse.grabbed = true;
        mouse.visible = false;
    }

    if key_input.pressed(&Key::Escape) {
        mouse.grabbed = false;
        mouse.visible = true;
    }
}

fn main() {
    env_logger::init();

    App::new()
        .set_runner(WinitRunner)
        .add_plugin(RenderPlugin)
        .add_plugin(DebugLinePlugin)
        .add_plugin(PbrPlugin)
        .add_plugin(TransformPlugin)
        .add_exclusive_system(StartupSystem)
        .add_system(camera_aspect_system.system())
        .add_system(window_capture_system.system())
        .register_component::<Rotate>()
        .register_component::<CameraRotate>()
        .run();
}
