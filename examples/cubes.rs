use ike::prelude::*;

fn main() {
    App::new()
        .add_resource(Gravity::default())
        .add_plugin(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(grab_cursor_system)
        .add_system(rotate_system)
        .add_system(move_camera_system)
        .add_system(spawn_sphere)
        .add_system(despawn_sphere)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..Default::default()
        })
        .insert(CameraRotate::default());

    commands.spawn().insert(DirectionalLightBundle {
        light: DirectionalLight {
            direction: Vec3::new(-1.0, -1.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mesh = shape::cube(1.0, 1.0, 1.0);

    let transform = Transform::from_xyz(0.0, -5.0, 0.0)
        .with_rotation(Quat::from_rotation_x(0.1))
        .with_scale(Vec3::new(20.0, 1.0, 20.0));

    commands.insert_resource(shape::uv_sphere(0.75, 32));

    commands
        .spawn()
        .insert(MaterialBundle {
            material: StandardMaterial::default(),
            mesh: mesh.clone(),
            transform,
            ..Default::default()
        })
        .insert(RigidBody::Kinematic)
        .insert(Collider::cuboid(Vec3::new(10.0, 0.5, 10.0)));
}

#[derive(Component)]
struct Rotate;

#[derive(Component, Default)]
struct CameraRotate {
    pub x: f32,
    pub y: f32,
}

fn spawn_sphere(mut commands: Commands, keyboard: Res<Input<Key>>, mesh: Res<Mesh>) {
    if !keyboard.is_pressed(&Key::E) {
        return;
    }

    commands
        .spawn()
        .insert(MaterialBundle {
            material: StandardMaterial {
                transmission: 1.0,
                ..Default::default()
            },
            mesh: mesh.clone(),
            ..Default::default()
        })
        .insert(RigidBody::default())
        .insert(Collider::sphere(0.75));
}

fn despawn_sphere(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform), With<RigidBody>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.length() > 20.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn grab_cursor_system(
    keyboard: Res<Input<Key>>,
    mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    let window = windows.primary();

    if mouse.is_pressed(&MouseButton::Left) {
        window.set_cursor_grabbed(true);
        window.set_cursor_visible(false);
    }

    if keyboard.is_pressed(&Key::Escape) {
        window.set_cursor_grabbed(false);
        window.set_cursor_visible(true);
    }
}

fn move_camera_system(
    keyboard: Res<Input<Key>>,
    mouse: Res<Mouse>,
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &mut CameraRotate), With<Camera>>,
) {
    let (mut transform, mut rotate) = query.iter_mut().next().unwrap();

    if windows.primary().is_cursor_grabbed() {
        rotate.x -= mouse.motion.x * 0.001;
        rotate.y -= mouse.motion.y * 0.001;

        transform.rotation = Quat::from_euler(EulerRot::default(), rotate.x, rotate.y, 0.0);

        let mut direction = Vec3::ZERO;

        if keyboard.is_held(&Key::W) {
            direction += transform.forward();
        }

        if keyboard.is_held(&Key::S) {
            direction -= transform.forward();
        }

        if keyboard.is_held(&Key::A) {
            direction -= transform.right();
        }

        if keyboard.is_held(&Key::D) {
            direction += transform.right();
        }

        if keyboard.is_held(&Key::Space) {
            direction += transform.up();
        }

        if keyboard.is_held(&Key::LShift) {
            direction -= transform.up();
        }

        transform.translation += direction.normalize_or_zero() * 0.1;
    }
}

fn rotate_system(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.1);
    }
}
