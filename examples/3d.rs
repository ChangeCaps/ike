use ike::prelude::*;

#[derive(Component)]
struct Rotate;

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}

fn setup(
    commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let mut camera_transform = Transform::from_xyz(3.0, 2.0, 2.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    commands
        .spawn()
        .insert(camera_transform)
        .insert(GlobalTransform::default())
        .insert(Camera::default());

    commands.spawn().insert(DirectionalLight {
        direction: Vec3::new(-0.5, -1.0, -0.5),
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::cube(Vec3::ONE / 2.0));
    let material = materials.add(PbrMaterial {
        ..Default::default()
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -1.0, 0.0).with_scale(Vec3::new(100.0, 0.1, 100.0)))
        .insert(GlobalTransform::default())
        .insert(mesh.clone())
        .insert(material.clone());

    for x in -2..=2 {
        for z in -2..=2 {
            commands
                .spawn()
                .insert(Transform::from_xyz(x as f32, 0.0, z as f32))
                .insert(GlobalTransform::default())
                .insert(mesh.clone())
                .insert(material.clone())
                .insert(Rotate);
        }
    }
}

fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.01);

        let y = f32::cos(transform.translation.x + time.seconds_since_startup())
            + f32::cos(transform.translation.z + time.seconds_since_startup());
        transform.translation.y = y;
    }
}
