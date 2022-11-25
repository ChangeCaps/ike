use ike::{lumi::LumiPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugin(LumiPlugin)
        .add_startup_system(setup)
        .add_system(rotate_system)
        .run();
}

#[derive(Component)]
struct Rotate;

fn setup(mut commands: Commands) {
    commands.spawn().insert(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 4.0),
        ..Default::default()
    });

    commands.spawn().insert(DirectionalLightBundle {
        light: DirectionalLight {
            direction: Vec3::new(-1.0, -1.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mesh = shape::cube(1.0, 1.0, 1.0);

    commands
        .spawn()
        .insert(MaterialBundle {
            material: StandardMaterial::default(),
            mesh: mesh.clone(),
            ..Default::default()
        })
        .insert(Rotate)
        .with_children(|parent| {
            parent
                .spawn()
                .insert(MaterialBundle {
                    material: StandardMaterial::default(),
                    mesh: mesh.clone(),
                    transform: Transform::from_xyz(2.0, 0.0, 0.0),
                    ..Default::default()
                })
                .insert(Rotate);
        });
}

fn rotate_system(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.1);
    }
}
