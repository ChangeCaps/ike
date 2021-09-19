use ike::{d3::SkyNode, prelude::*};

struct GameState {
    mesh: Mesh,
    transform: Transform3d,
    camera_parent: Transform3d,
    camera: PerspectiveCamera,
    camera_rot: Vec2,
    light: PointLight,
    font: Font,
    scene: PbrScene,
    time: f32,
    sky_texture: Texture,
    num: i32,
    agent: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut camera = PerspectiveCamera::default();

        camera.transform.translation = Vec3::new(0.0, 0.0, -5.0);
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
            camera_rot: Vec2::new(0.4, 0.3),
            font: Font::load("assets/font.ttf", 100.0).unwrap(),
            scene: PbrScene::load_gltf("assets/sponza/Sponza.gltf").unwrap(),
            time: 0.0,
            sky_texture: Texture::load("assets/hdr.png").unwrap(),
            num: 3,
            agent: true,
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
            self.camera_rot.x -= ctx.mouse.delta().x * 0.001;
            self.camera_rot.y += ctx.mouse.delta().y * 0.001;
        }

        self.camera_parent.rotation = Quat::from_rotation_y(self.camera_rot.x);
        self.camera_parent.rotation *= Quat::from_rotation_x(self.camera_rot.y);

        self.camera.transform(&self.camera_parent);

        self.camera.transform.translation -=
            self.camera.transform.local_z() * ctx.mouse.wheel_delta.y * 0.2;

        if ctx.key_input.pressed(&Key::Left) {
            self.num -= 1;
        }

        if ctx.key_input.pressed(&Key::Right) {
            self.num += 1;
        }

        if ctx.key_input.pressed(&Key::A) {
            self.agent ^= true;
        }
    }

    #[inline]
    fn render(&mut self, ctx: &mut UpdateCtx) {
        for x in -self.num..=self.num {
            for z in -self.num..=self.num {
                let y = (x as f32 + z as f32 * 0.3 + self.time).sin() * 0.3;
                let transform = Transform3d::from_xyz(x as f32 * 2.0, y, z as f32 * 2.0);

                if self.agent {
                    let scene = self.scene.transform(&transform);
                    /*
                    scene
                        .animate("Walk", (self.time + x as f32 * 0.4 + z as f32 * 0.1) % 1.0)
                        .unwrap();
                        */
                    ctx.draw(&scene);
                } else {
                    ctx.draw(&self.mesh.transform(&transform));
                }
            }
        }

        ctx.draw(&self.light);

        let fps_transform = Transform3d::from_xyz(0.0, 1.2, 0.0);

        let mut text = TextSprite::new(&self.font, fps_transform);
        text.text = format!("{:2.2} fps | {} objects", 1.0 / ctx.delta_time, (self.num * 2).pow(2)).into();
        text.size = 0.5;

        ctx.draw(&text);

        ctx.draw(&SkyTexture::new(&self.sky_texture));

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

    main_pass.push(SkyNode::default());
    main_pass.push(DebugNode::default());
    main_pass.push(D3Node::default());
    main_pass.push(SpriteNode2d::new());

    app.renderer.push(main_pass);

    app.run(GameState::new());
}
