use ike::prelude::*;
use ike_assets::AssetServer;

#[derive(Component, Default)]
pub struct Player {
    pub camera_angle: Vec2,
}

#[derive(Component)]
pub struct Garbage;

#[node]
impl Player {
    fn update(&mut self, node: Node) {
        let mut window = node.resource_mut::<Window>();
        let key_input = node.resource::<Input<Key>>();
        let mouse_input = node.resource::<Input<MouseButton>>();
        let time = node.resource::<Time>();
        let task_pool = node.resource::<TaskPool>();
        let mut transform = node.component_mut::<Transform>();
        let mut rigid_body = node.component_mut::<RigidBody>();

        if window.cursor_locked() {
            self.camera_angle -= window.cursor_delta() * 0.002;
            self.camera_angle.y = self.camera_angle.y.clamp(-1.3, 1.3);
        }

        if key_input.pressed(&Key::Escape) {
            window.set_cursor_locked(false);
            window.set_cursor_visible(true);
        }

        if mouse_input.pressed(&MouseButton::Left) {
            window.set_cursor_locked(true);
            window.set_cursor_visible(false);
        }

        transform.rotation = Quat::from_rotation_y(self.camera_angle.x);

        let vel_y = rigid_body.linear_velocity.y;
        rigid_body.linear_velocity = Vec3::ZERO;

        if key_input.held(&Key::W) {
            rigid_body.linear_velocity -= transform.local_z();
        }

        if key_input.held(&Key::S) {
            rigid_body.linear_velocity += transform.local_z();
        }

        if key_input.held(&Key::A) {
            rigid_body.linear_velocity -= transform.local_x();
        }

        if key_input.held(&Key::D) {
            rigid_body.linear_velocity += transform.local_x();
        }

        rigid_body.linear_velocity = rigid_body.linear_velocity.normalize_or_zero() * 5.0;
        rigid_body.linear_velocity.y = vel_y;

        let child = node.child(0);

        let child_global_transform = child.component::<GlobalTransform>();
        let mut child_transform = child.component_mut::<Transform>();
        child_transform.rotation = Quat::from_rotation_x(self.camera_angle.y);

        if mouse_input.held(&MouseButton::Right) {
            node.query_filter::<(&GlobalTransform, &mut RigidBody), With<Garbage>>()
                .par_for_each_mut(&task_pool, |(transform, mut rigid_body)| {
                    let target = child_global_transform.translation
                        - child_global_transform.local_z() * 10.0;

                    let diff = target - transform.translation;
                    let dist = diff.length().max(0.1);

                    rigid_body.linear_velocity +=
                        diff * (1.0 / dist.powi(2)) * time.delta_seconds() * 100.0;
                });
        }
    }
}

fn spawn_player(commands: &Commands) {
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .insert(RigidBody::dynamic())
        .insert(BoxCollider::new(Vec3::ONE))
        .insert(Player::default())
        .with_children(|parent| {
            parent
                .spawn()
                .insert(Transform::from_xyz(0.0, 1.0, 0.0))
                .insert(GlobalTransform::default())
                .insert(Camera::default());
        });
}

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .register_node::<Player>()
        .add_startup_system(setup)
        .run();
}

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let mut camera_transform = Transform::from_xyz(4.0, 3.0, 3.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    commands
        .spawn()
        .insert(DirectionalLight {
            direction: Vec3::new(-0.3, -1.0, -0.3),
            ..Default::default()
        })
        .insert(Transform::default());

    let image = asset_server.load_from_bytes(include_bytes!("../assets/rock.jpg") as &[u8], "jpg");

    let mesh = meshes.add(Mesh::cube(Vec3::ONE));
    let material = materials.add(PbrMaterial {
        base_color_texture: Some(image),
        ..Default::default()
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -1.0, 0.0).with_scale(Vec3::new(100.0, 1.0, 100.0)))
        .insert(mesh.clone())
        .insert(material.clone())
        .insert(RigidBody::kinematic())
        .insert(BoxCollider::new(Vec3::ONE));

    spawn_player(&commands);

    for x in -5..=5 {
        for y in -5..=5 {
            for z in -5..=5 {
                commands
                    .spawn()
                    .insert(
                        Transform::from_xyz(x as f32 * 0.5, y as f32 * 0.5 + 10.0, z as f32 * 0.5)
                            .with_scale(Vec3::ONE / 2.2),
                    )
                    .insert(mesh.clone())
                    .insert(material.clone())
                    .insert(RigidBody::dynamic())
                    .insert(BoxCollider::new(Vec3::ONE))
                    .insert(Garbage);
            }
        }
    }
}
