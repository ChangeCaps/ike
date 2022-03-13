use ike::prelude::*;

#[derive(Default)]
pub struct PlayerBuilder {
    pub transform: Transform,
    pub height: f32,
}

impl PlayerBuilder {
    pub fn new() -> Self {
        Self {
            height: 2.0,
            ..Default::default()
        }
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn spawn(self, commands: &Commands) {
        commands
            .spawn()
            .insert(self.transform)
            .insert(Player::default())
            .insert(Collider::capsule(Vec3::ZERO, Vec3::Y * self.height, 0.3))
            .insert(RigidBody::dynamic().with_rotation_lock(true))
            .with_children(|parent| {
                parent
                    .spawn()
                    .insert(Transform::from_xyz(0.0, self.height, 0.0))
                    .insert(Camera::default());
            });
    }
}

#[derive(Component, Default)]
pub struct Player {
    pub look_angle: Vec2,
}

#[node]
impl Player {
    fn update(&mut self, node: Node) {
        self.update_camera(&node);
        self.walk(&node);
    }
}

impl Player {
    fn update_camera(&mut self, node: &Node) {
        let window = &mut *node.resource_mut::<Window>();
        let key_input = node.resource::<Input<Key>>();
        let mouse_input = node.resource::<Input<MouseButton>>();

        if mouse_input.pressed(&MouseButton::Left) {
            window.set_cursor_visible(false);
            window.set_cursor_locked(true);
        }

        if key_input.pressed(&Key::Escape) {
            window.set_cursor_visible(true);
            window.set_cursor_locked(false);
        }

        // if cursor is captured, we should move the camera
        if window.cursor_locked() {
            self.look_angle -= window.cursor_delta() * 0.001;
            window.set_cursor_position(window.size() / 2.0);
        }

        let mut transform = node.component_mut::<Transform>();
        transform.rotation = Quat::from_rotation_y(self.look_angle.x);

        let child = node.child(0);
        let mut transform = child.component_mut::<Transform>();
        transform.rotation = Quat::from_rotation_x(self.look_angle.y);
    }

    fn walk(&mut self, node: &Node) {
        let key_input = node.resource::<Input<Key>>();
        let time = node.resource::<Time>();

        let transform = node.component::<Transform>();
        let is_grounded = node
            .cast_ray_length(transform.translation, -Vec3::Y, 0.5)
            .is_some();

        let mut rigid_body = node.component_mut::<RigidBody>();

        let mut movement = Vec3::ZERO;

        if key_input.held(&Key::W) {
            movement -= transform.local_z();
        }

        if key_input.held(&Key::S) {
            movement += transform.local_z();
        }

        if key_input.held(&Key::A) {
            movement -= transform.local_x();
        }

        if key_input.held(&Key::D) {
            movement += transform.local_x();
        }

        movement.y = 0.0;

        let target_velocity = movement * 5.0;
        let velocity = rigid_body.linear_velocity;

        let diff = target_velocity - velocity;
        let dist = Vec3::distance(velocity, target_velocity);

        let max_step = dist.min(time.delta_seconds() * 50.0);

        if is_grounded && diff.length() > 0.0 {
            rigid_body.linear_velocity += diff.normalize() * max_step;
        }
    }
}
