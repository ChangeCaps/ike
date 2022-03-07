use ike::prelude::*;

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(rotate.system())
        .run();
}

fn setup(
    commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let mut camera_transform = Transform::from_xyz(2.0, 1.0, 1.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    commands
        .spawn()
        .insert(camera_transform)
        .insert(GlobalTransform::default())
        .insert(Camera::default());

    commands.spawn().insert(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::cube(Vec3::ONE / 2.0));
    let material = materials.add(PbrMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    for x in -2..=2 {
        for z in -2..=2 {
            commands
                .spawn()
                .insert(Transform::from_xyz(x as f32, 0.0, z as f32))
                .insert(GlobalTransform::default())
                .insert(mesh.clone())
                .insert(material.clone());
        }
    }
}

fn rotate(mut query: Query<&mut Transform, With<Handle<Mesh>>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.01);
    }
}
