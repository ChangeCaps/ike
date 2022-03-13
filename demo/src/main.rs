mod launcher;
mod player;

use ike::prelude::*;
use launcher::{Catcher, Launcher, LauncherBuilder, Sphere};
use player::{Player, PlayerBuilder};

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .register_node::<Player>()
        .register_node::<Launcher>()
        .register_node::<Catcher>()
        .register_node::<Sphere>()
        .add_startup_system(setup)
        .run();
}

fn setup(
    commands: Commands,
    assert_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let cube_mesh = meshes.add(Mesh::cube(Vec3::ONE));
    let floor_material = materials.add(PbrMaterial::default());

    let launcher_mesh: Handle<GltfMesh> = assert_server.load("assets/launcher.mesh.glb");
    assert_server.load_untyped("assets/sphere.mesh.glb");

    // spawn floor
    commands
        .spawn()
        .insert(Transform::from_scale(Vec3::new(100.0, 0.5, 100.0)))
        .insert(cube_mesh)
        .insert(floor_material)
        .insert(RigidBody::kinematic())
        .insert(Collider::cube(Vec3::new(100.0, 0.5, 100.0)));

    // spawn sun
    commands.spawn().insert(DirectionalLight {
        illuminance: 200.0,
        direction: Vec3::new(-0.2, -1.0, -0.5),
        ..Default::default()
    });

    // spawn launchers

    for z in -5..=5 {
        let side = if z % 2 == 0 { 1.0 } else { -1.0 };

        LauncherBuilder::new()
            .transform(
                Transform::from_xyz(-15.0 * side, 1.0, z as f32 * 4.0)
                    .with_rotation(Quat::from_rotation_z(-45.0f32.to_radians() * side)),
            )
            .spawn(&commands);

        commands
            .spawn()
            .insert(
                Transform::from_xyz(15.0 * side, 1.0, z as f32 * 4.0)
                    .with_rotation(Quat::from_rotation_z(45.0f32.to_radians() * side)),
            )
            .insert(launcher_mesh.clone())
            .insert(Catcher)
            .insert(RigidBody::kinematic())
            .insert(Collider::sphere(1.0));
    }

    // spawn player
    PlayerBuilder::new()
        .transform(Transform::from_xyz(0.0, 1.0, 0.0))
        .spawn(&commands);
}
