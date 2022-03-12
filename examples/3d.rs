use ike::prelude::*;
use ike_assets::AssetServer;

#[derive(Component, Default)]
pub struct Player {
    pub camera_angle: Vec2,
    pub selected: Option<Entity>,
}

#[derive(Component)]
pub struct GravityPoint(f32);

pub struct Materials {
    pub selected: Handle<PbrMaterial>,
    pub rock: Handle<PbrMaterial>,
}

#[derive(Component)]
pub struct Garbage;

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

            let size = window.size();
            window.set_cursor_position(size / 2.0);
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

        if let Some(hit) = node.cast_ray(
            child_global_transform.translation,
            -child_global_transform.local_z(),
            None,
        ) {
            let materials = node.resource::<Materials>();

            if let Some(selected) = self.selected {
                let selected_node = node.get_node(&selected);
                selected_node.insert(materials.rock.clone());
            }

            let hit_node = node.get_node(&hit.entity);
            hit_node.insert(materials.selected.clone());

            self.selected = Some(hit.entity);
        } else {
            let materials = node.resource::<Materials>();

            if let Some(selected) = self.selected.take() {
                let selected_node = node.get_node(&selected);
                selected_node.insert(materials.rock.clone());
            }
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
        .add_system(gravity_point_system)
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
        base_color_texture: Some(image.clone()),
        ..Default::default()
    });

    let selected = materials.add(PbrMaterial {
        base_color_texture: Some(image),
        emission: Color::rgb(0.7, 0.1, 0.0) * 10.0,
        ..Default::default()
    });

    commands.insert_resource(Materials {
        selected,
        rock: material.clone(),
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -1.0, 0.0).with_scale(Vec3::new(100.0, 1.0, 100.0)))
        .insert(mesh.clone())
        .insert(material.clone())
        .insert(RigidBody::kinematic())
        .insert(BoxCollider::new(Vec3::ONE));

    spawn_player(&commands);

    for x in -3..=3 {
        for y in -3..=3 {
            for z in -3..=3 {
                commands
                    .spawn()
                    .insert(
                        Transform::from_xyz(
                            x as f32 * 1.0 + 5.0,
                            y as f32 * 1.0 + 20.0,
                            z as f32 * 1.0,
                        )
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

fn gravity_point_system(
    mut garbage_query: Query<(Entity, &GlobalTransform, &mut RigidBody), With<Garbage>>,
    gravity_point_query: Query<(Entity, &GlobalTransform, &GravityPoint)>,
    time: Res<Time>,
) {
    for (entity, transform, gravity) in gravity_point_query.iter() {
        for (garbage_entity, garbage_transform, mut garbage_rigid_body) in garbage_query.iter_mut()
        {
            if entity == garbage_entity {
                continue;
            }

            let diff = transform.translation - garbage_transform.translation;
            let dist = diff.length().max(0.5);
            let force = diff.normalize() * (1.0 / dist.powi(2)) * gravity.0 * time.delta_seconds();
            garbage_rigid_body.linear_velocity += force;
        }
    }
}
