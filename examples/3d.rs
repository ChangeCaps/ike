use ike::prelude::*;

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(rotate.system())
        .run();
}

fn setup(commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut camera_transform = Transform::from_xyz(2.0, 1.0, 1.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    commands
        .spawn()
        .insert(camera_transform)
        .insert(GlobalTransform::default())
        .insert(Camera::default());

    let mesh = meshes.add(Mesh::cube(Vec3::ONE / 2.0));

    for x in -10..=10 {
        for z in -10..=10 {
            commands
                .spawn()
                .insert(Transform::from_xyz(x as f32, 0.0, z as f32))
                .insert(GlobalTransform::default())
                .insert(mesh.clone());
        }
    }
}

fn rotate(mut query: Query<&mut Transform, With<Handle<Mesh>>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.1);
    }
}
