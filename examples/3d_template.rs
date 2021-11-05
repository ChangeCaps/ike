use ike::prelude::*;
use ike_egui::{egui, EguiPlugin};
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

#[derive(Clone, PartialEq, Eq)]
enum MoveMode {
    Wave,
}

#[derive(Default)]
struct MoveOptions {
    move_mode: Option<MoveMode>,
}

struct Move;

impl Component for Move {
    fn update(&mut self, node: &mut Node<'_>, world: &World) {
        let key_input = world.read_resource::<Input<Key>>().unwrap();

        if key_input.down(&Key::Up) {
            let transform = &mut *node.get_component_mut::<Transform>().unwrap();
            transform.translation += transform.local_y() * 0.1;
        }

        if key_input.down(&Key::Down) {
            let transform = &mut *node.get_component_mut::<Transform>().unwrap();
            transform.translation -= transform.local_y() * 0.1;
        }

        let time = world.read_resource::<Time>().unwrap();
        let move_options = world.read_resource::<MoveOptions>().unwrap();

        let t = time.time_since_startup();

        match move_options.move_mode {
            Some(MoveMode::Wave) => {
                let transform = &mut *node.get_component_mut::<Transform>().unwrap();

                transform.translation.y = (t + transform.translation.x / 5.0).sin()
                    * (t * 0.2 + transform.translation.z).sin();
            }
            None => {}
        }
    }
}

struct Material(Handle<PbrMaterial>, Handle<Mesh>);

fn setup(
    commands: Commands,
    mut main_camera: ResMut<MainCamera>,
    mut envs: ResMut<Assets<Environment>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let hdr = HdrTexture::load("assets/env.hdr").unwrap();

    let mut env = Environment::default();
    env.load(&hdr);

    let env = envs.add(env);
    commands.insert_resource(env);

    let node = commands.spawn_node("Camera");

    let mut transform = Transform::from_xyz(2.0, 3.0, 1.0);
    transform.look_at(Vec3::ZERO, Vec3::Y);

    node.insert(PerspectiveProjection::default());
    node.insert(transform);
    node.insert(CameraRotate(Vec2::ZERO));

    main_camera.0 = Some(node.entity());

    let light = commands.spawn_node("light");

    light.insert(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::cube(Vec3::ONE / 2.0));

    let material = materials.add(PbrMaterial {
        roughness: 0.01,
        metallic: 0.01,
        reflectance: 0.5,
        ..PbrMaterial::default()
    });

    let node = commands.spawn_node("rotate");

    node.insert(Transform::from_scale(Vec3::new(20.0, 0.5, 20.0)));
    node.insert(mesh.clone());
    node.insert(material.clone());
    node.insert(RigidBody::kinematic());
    node.insert(BoxCollider::new(Vec3::ONE / 2.0));

    commands.insert_resource(Material(material.clone(), mesh.clone()));
}

fn camera_aspect_system(window: Res<Window>, query: Query<&mut PerspectiveProjection>) {
    let aspect = window.aspect();

    for mut projection in query {
        projection.aspect = aspect;
    }
}

fn window_capture_system(mut mouse: ResMut<Mouse>, key_input: Res<Input<Key>>) {
    if key_input.pressed(&Key::Space) {
        mouse.grabbed = true;
        mouse.visible = false;
    }

    if key_input.pressed(&Key::Escape) {
        mouse.grabbed = false;
        mouse.visible = true;
    }
}

fn egui_system(
    ctx: Res<egui::CtxRef>,
    time: Res<Time>,
    material: Res<Material>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut move_options: ResMut<MoveOptions>,
) {
    egui::SidePanel::left("move_panel").show(&ctx, |ui| {
        ui.heading("Move");

        ui.selectable_value(&mut move_options.move_mode, None, "None");
        ui.selectable_value(&mut move_options.move_mode, Some(MoveMode::Wave), "Wave");
    });

    egui::Window::new("Performance").show(&ctx, |ui| {
        ui.label(format!("fps: {}", time.frames_per_second()));
    });

    egui::Window::new("Material").show(&ctx, |ui| {
        let material = materials.get_mut(&material.0).unwrap();

        egui::Grid::new("material_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("roughness");
                ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0));
                ui.end_row();

                ui.label("metallic");
                ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
                ui.end_row();

                ui.label("reflectance");
                ui.add(egui::Slider::new(&mut material.reflectance, 0.0..=1.0));
                ui.end_row();

                ui.label("albedo");
                let mut color = [material.albedo.r, material.albedo.g, material.albedo.b];
                ui.color_edit_button_rgb(&mut color);
                material.albedo.r = color[0];
                material.albedo.g = color[1];
                material.albedo.b = color[2];
                ui.end_row();
            });
    });
}

fn spawn_system(commands: Commands, material: Res<Material>) {
    let node = commands.spawn_node("cube");

    node.insert(Transform::from_xyz(0.0, 10.0, 0.0));
    node.insert(RigidBody::default());
    node.insert(BoxCollider::new(Vec3::ONE / 2.0));
    node.insert(material.0.clone());
    node.insert(material.1.clone());
}

fn main() {
    env_logger::init();

    App::new()
        .set_runner(WinitRunner)
        .add_plugin(RenderPlugin)
        .add_plugin(DebugLinePlugin)
        .add_plugin(PbrPlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PhysicsPlugin)
        .init_resource::<MoveOptions>()
        .add_startup_system(setup.system())
        .add_system(camera_aspect_system.system())
        .add_system(window_capture_system.system())
        .add_system(egui_system.system())
        .add_system(spawn_system.system())
        .register_component::<Rotate>()
        .register_component::<CameraRotate>()
        .register_component::<Move>()
        .run();
}
