use ike::prelude::*;
use ike_core::WorldRef;
use ike_egui::{egui, EguiPlugin};
use ike_physics::RigidBodies;

struct Rotate;

impl Component for Rotate {
    fn update(&mut self, node: &mut Node, _world: &WorldRef) {
        let mut transform = node.get_component_mut::<Transform>().unwrap();

        transform.rotation *= Quat::from_rotation_y(0.01);
    }
}

struct CameraRotate(Vec2);

impl Component for CameraRotate {
    fn update(&mut self, node: &mut Node, world: &WorldRef) {
        let mouse = world.get_resource::<Mouse>().unwrap();

        if mouse.grabbed {
            self.0 += mouse.movement * 0.001 * -1.0;
        }

        let key_input = world.get_resource::<Input<Key>>().unwrap();

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
    is_kinematic: bool,
    spawn: String,
}

#[derive(Reflect)]
struct Move;

impl Component for Move {
    fn update(&mut self, node: &mut Node, world: &WorldRef) {
        let time = world.get_resource::<Time>().unwrap();
        let key_input = world.get_resource::<Input<Key>>().unwrap();

        if let Some(mut rb) = node.get_component_mut::<RigidBody>() {
            let transform = node.get_component::<GlobalTransform>().unwrap();

            if key_input.down(&Key::Up) {
                rb.linear_velocity += transform.translation.normalize() * time.delta_time() * 50.0;
            }

            if key_input.down(&Key::Down) {
                rb.linear_velocity -= transform.local_z() * time.delta_time() * 50.0;
            }

            if transform.translation.y < -10.0 {
                world.despawn(&node.entity());
            }
        } else {
            let transform = &mut *node.get_component_mut::<Transform>().unwrap();

            if key_input.down(&Key::Up) {
                transform.translation += transform.local_y() * time.delta_time() * 50.0;
            }

            if key_input.down(&Key::Down) {
                transform.translation -= transform.local_y() * time.delta_time() * 50.0;
            }
        }

        let transform = node.get_component::<Transform>().unwrap();

        DebugLine::new()
            .from(transform.translation)
            .to(transform.translation + transform.local_y())
            .use_depth()
            .draw();

        let move_options = world.get_resource::<MoveOptions>().unwrap();

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
    mut envs: ResMut<Assets<Environment>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let hdr = HdrTexture::load("assets/hdr.hdr").unwrap();

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
    node.insert(MainCamera);

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

fn egui_spawn_system(
    ctx: Res<egui::CtxRef>,
    mut move_options: ResMut<MoveOptions>,
    type_registry: Res<TypeRegistry>,
    world: WorldRef,
    query: Query<&mut RigidBody, With<Move>>,
    move_query: Query<Entity, With<Move>>,
) {
    egui::SidePanel::left("move_panel").show(&ctx, |ui| {
        ui.heading("Move");

        ui.selectable_value(&mut move_options.move_mode, None, "None");
        ui.selectable_value(&mut move_options.move_mode, Some(MoveMode::Wave), "Wave");
        let response = ui.checkbox(&mut move_options.is_kinematic, "kinematic");

        if response.changed() {
            for mut rigid_body in query {
                rigid_body.kinematic = move_options.is_kinematic;
            }
        }

        ui.text_edit_singleline(&mut move_options.spawn);

        if ui.button("Spawn").clicked() {
            if let Ok(mut scene) = ike_gltf::load_gltf(&move_options.spawn, &world) {
                for node in scene.entities.values_mut() {
                    node.insert(Move);
                }

                scene.spawn(world.commands(), &type_registry);

                for entity in move_query {
                    world.despawn(&entity);
                }
            }
        }
    });
}

fn egui_system(
    ctx: Res<egui::CtxRef>,
    time: Res<Time>,
    material: Res<Material>,
    rigid_bodies: Res<RigidBodies>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    egui::Window::new("Performance").show(&ctx, |ui| {
        egui::Grid::new("performance_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("fps");
                ui.label(format!("{}", time.frames_per_second()));
                ui.end_row();

                ui.label("rigid bodies");
                ui.label(format!("{}", rigid_bodies.0.len()));
                ui.end_row();
            });
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

fn spawn_system(
    commands: Commands,
    material: Res<Material>,
    key_input: Res<Input<Key>>,
    move_options: Res<MoveOptions>,
) {
    if key_input.pressed(&Key::L) {
        let node = commands.spawn_node("cube");

        let collider = BoxCollider::new(Vec3::ONE / 2.0);
        //collider.debug = Some(Color::GREEN);

        let mut rb = RigidBody::default();
        rb.kinematic = move_options.is_kinematic;
        rb.continuous = true;

        node.insert(Transform::from_xyz(0.0, 10.0, 0.0));
        node.insert(rb);
        node.insert(collider);
        node.insert(Move);
        node.insert(material.0.clone());
        node.insert(material.1.clone());
    }
}

#[ike::main]
fn main(app: &mut AppBuilder) {
    env_logger::init();

    app.add_plugin(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .init_resource::<MoveOptions>()
        .add_startup_system(setup.system())
        .add_system(camera_aspect_system.system())
        .add_system(window_capture_system.system())
        .add_system(egui_system.system())
        .add_system(spawn_system.system())
        .add_system(egui_spawn_system.system())
        .register_component::<Rotate>()
        .register_component::<CameraRotate>()
        .register_component::<Move>()
        .register::<Move>();
}
