use ike::prelude::*;

#[derive(Component, Default)]
pub struct Player {
    pub camera_angle: Vec2,
}

#[node]
impl Player {
    fn update(&mut self, node: Node) {
        let mut window = node.resource_mut::<Window>();
        let key_input = node.resource::<Input<Key>>();
        let mouse_input = node.resource::<Input<MouseButton>>();
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

        let child = node.child(0);

        let mut child_transform = child.component_mut::<Transform>();
        child_transform.rotation = Quat::from_rotation_x(self.camera_angle.y);
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

    let mesh = meshes.add(Mesh::cube(Vec3::ONE));
    let material = materials.add(PbrMaterial {
        ..Default::default()
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -1.0, 0.0).with_scale(Vec3::new(100.0, 0.2, 100.0)))
        .insert(mesh.clone())
        .insert(material.clone())
        .insert(RigidBody::kinematic())
        .insert(BoxCollider::new(Vec3::ONE));

    spawn_player(&commands);

    for y in 5..20 {
        commands
            .spawn()
            .insert(Transform::from_xyz(0.0, y as f32 * 1.5, 0.0))
            .insert(mesh.clone())
            .insert(material.clone())
            .insert(RigidBody::dynamic())
            .insert(BoxCollider::new(Vec3::ONE));
    }
}
