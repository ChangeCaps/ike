use ike::prelude::*;

struct GameState {
    mesh: Mesh,
    transform: Transform3d,
    camera: PerspectiveCamera,
    light: PointLight,
}

impl GameState {
    pub fn new() -> Self {
        let mut camera = PerspectiveCamera::default();

        camera.transform.translation = Vec3::new(3.0, 2.0, 1.5);
        camera.transform.look_at(Vec3::ZERO, Vec3::Y);

        Self {
            mesh: Mesh::cube(Vec3::ONE),
            transform: Transform3d::IDENTITY,
            camera,
            light: PointLight {
                position: Vec3::new(1.0, 2.0, 2.0),
                color: Color::WHITE,
                intensity: 200.0,
                range: 20.0,
                radius: 0.0,
            },
        }
    }
}

impl State for GameState {
    #[inline]
    fn render(&mut self, ctx: &mut UpdateCtx) {
        self.transform.rotation *= Quat::from_rotation_x(ctx.delta_time);

        for x in -5..=5 {
            for z in -5..=5 {
                let transform =
                    Transform3d::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0) * &self.transform;

                ctx.draw(&self.mesh.transform(&transform));
            }
        }

        ctx.draw(&self.light);

        let mut debug = DebugMesh::with_transform(&self.mesh, &self.transform);
        debug.face_normals = Some(Color::RED);
        debug.vertex_normals = Some(Color::GREEN);
        debug.depth = true;

        ctx.draw(&debug);

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
