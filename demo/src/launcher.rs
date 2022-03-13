use ike::prelude::*;

use rand::random;

#[derive(Default)]
pub struct LauncherBuilder {
    pub transform: Transform,
}

impl LauncherBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn spawn(self, commands: &Commands) {
        let mesh = Handle::<GltfMesh>::new("assets/launcher.mesh.glb");

        commands
            .spawn()
            .insert(self.transform)
            .insert(Launcher::default())
            .insert(GlobalTransform::default())
            .insert(mesh);
    }
}

#[derive(Component, Default)]
pub struct Launcher {
    pub timer: f32,
}

#[node]
impl Launcher {
    fn update(&mut self, node: Node) {
        let time = node.resource::<Time>();
        self.timer += time.delta_seconds();

        if self.timer > 0.05 {
            self.timer = 0.0;

            let transform = node.component::<Transform>();

            let mut rigid_body = RigidBody::dynamic();
            rigid_body.linear_velocity = transform.local_y() * 20.0;

            let mesh = Handle::<GltfMesh>::new("assets/sphere.mesh.glb");

            node.spawn()
                .insert(Sphere)
                .insert(transform.clone().with_scale(Vec3::ONE * 0.2))
                .insert(mesh)
                .insert(rigid_body)
                .insert(Collider::sphere(0.2));
        }
    }
}

pub fn random_circle(radius: f32) -> Vec2 {
    // NOTE: this is actually faster than doing angle-distance
    loop {
        let point = Vec2::new(random(), random()) * 2.0 - 1.0;

        if point.length_squared() <= 1.0 {
            break point * radius;
        }
    }
}

#[derive(Component)]
pub struct Sphere;
