use std::borrow::Cow;

use ike::prelude::*;

struct GameState {
    mesh: Mesh,
    transform: Transform3d,
    camera_parent: Transform3d,
    camera: PerspectiveCamera,
    light: PointLight,
    texture: Texture,
    normal_map: Texture,
    scene: PbrScene,
    time: f32,
}

impl GameState {
    pub fn new() -> Self {
        let mut camera = PerspectiveCamera::default();

        camera.transform.translation = Vec3::new(3.0, 2.0, 1.5) * 3.0;
        camera.transform.look_at(Vec3::ZERO, Vec3::Y);

        Self {
            //mesh: Mesh::cube(Vec3::ONE),
            mesh: Mesh::sphere(0.5, 20, 20),
            transform: Transform3d::IDENTITY,
            camera,
            camera_parent: Transform3d::IDENTITY,
            light: PointLight {
                position: Vec3::new(1.0, 2.0, 2.0),
                color: Color::WHITE,
                intensity: 300.0,
                range: 50.0,
                radius: 0.0,
            },
            texture: Texture::load("assets/brick.jpg").unwrap(),
            normal_map: Texture::load("assets/brick_normal.png").unwrap(),
            scene: PbrScene::load_gltf("assets/agent.glb").unwrap(),
            time: 0.0,
        }
    }
}

impl State for GameState {
    #[inline]
    fn start(&mut self, _ctx: &mut StartCtx) {
        self.mesh.calculate_tangents();
    }

    #[inline]
    fn update(&mut self, ctx: &mut UpdateCtx) {
        self.time += ctx.delta_time;

        self.transform.rotation *= Quat::from_rotation_x(ctx.delta_time);

        if ctx.mouse_input.down(&MouseButton::Middle) {
            self.camera_parent.rotation *= Quat::from_rotation_y(-ctx.mouse.delta().x * 0.001);
        } else {
            self.camera_parent.rotation *= Quat::from_rotation_y(ctx.delta_time * 0.3);
        }

        self.camera.transform(&self.camera_parent);

        self.camera.transform.translation -=
            self.camera.transform.local_z() * ctx.mouse.wheel_delta.y * 0.2;
    }

    #[inline]
    fn render(&mut self, ctx: &mut UpdateCtx) {
        for x in -20..=20 {
            for z in -20..=20 {
                let transform = Transform3d::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0);

                let mut scene = self.scene.transform(&transform);
                scene.animate(0, (self.time * 0.2 + x as f32 * 0.4 + z as f32 * 0.1) % 0.95);
                ctx.draw(&scene);
            }
        }        
        
        ctx.draw(&self.light);

        self.camera.projection.scale(ctx.window.size);
        ctx.views.render_main_view(self.camera.camera());
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    let mut app = App::new();

    let mut main_pass = MainPass::default();
    main_pass.sample_count = 4;
    main_pass.clear_color = Color::BLUE;

    let mut main_pass = Pass::new(main_pass);

    main_pass.push(DebugNode::default());
    main_pass.push(D3Node::default());

    app.renderer.push(main_pass);

    app.run(GameState::new());
}
