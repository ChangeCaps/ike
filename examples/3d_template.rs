use ike::{cube_texture::CubeTexture, d3::SkyNode, prelude::*};

struct GameState {
    mesh: Mesh,
    transform: Transform3d,
    camera: PerspectiveCamera,
    camera_rot: Vec2,
    light: PointLight,
    font: Font,
    scene: PbrScene,
    time: f32,
    sky_texture: HdrTexture,
    env: Environment,
    num: i32,
    agent: bool,
    ground: Mesh,
    metal: PbrMaterial,
}

impl GameState {
    pub fn new() -> Self {
        let mut camera = PerspectiveCamera::default();

        camera.transform.translation = Vec3::new(0.0, 1.0, 2.0);
        camera.transform.look_at(Vec3::ZERO, Vec3::Y);

        Self {
            mesh: Mesh::sphere(0.5, 20, 20),
            transform: Transform3d::IDENTITY,
            camera,
            light: PointLight {
                position: Vec3::new(0.0, 2.0, 0.0),
                color: Color::WHITE,
                intensity: 300.0,
                range: 50.0,
                radius: 0.0,
            },
            camera_rot: Vec2::ZERO,
            font: Font::load("assets/font.ttf", 100.0).unwrap(),
            scene: PbrScene::load_gltf("assets/wa.glb").unwrap(),
            time: 0.0,
            sky_texture: HdrTexture::load("assets/hdr.hdr").unwrap(),
            env: Environment::default(),
            num: 0,
            agent: true,
            ground: Mesh::plane(Vec2::splat(500.0)),
            metal: PbrMaterial::metal(),
        }
    }
}

impl State for GameState {
    #[inline]
    fn start(&mut self, ctx: &mut StartCtx) {
        self.mesh.calculate_tangents();

        self.env.load(ctx.render_ctx, &self.sky_texture);
    }

    #[inline]
    fn update(&mut self, ctx: &mut UpdateCtx) {
        self.time += ctx.delta_time;

        self.transform.rotation *= Quat::from_rotation_x(ctx.delta_time);

        if ctx.window.cursor_grab {
            self.camera_rot.x -= ctx.mouse.movement.x * 0.001;
            self.camera_rot.y -= ctx.mouse.movement.y * 0.001;
        }

        self.camera.transform.rotation = Quat::from_rotation_y(self.camera_rot.x);
        self.camera.transform.rotation *= Quat::from_rotation_x(self.camera_rot.y);

        if ctx.key_input.down(&Key::W) {
            self.camera.transform.translation -=
                self.camera.transform.local_z() * ctx.delta_time * 2.0;
        }

        if ctx.key_input.down(&Key::S) {
            self.camera.transform.translation +=
                self.camera.transform.local_z() * ctx.delta_time * 2.0;
        }

        if ctx.key_input.down(&Key::A) {
            self.camera.transform.translation -=
                self.camera.transform.local_x() * ctx.delta_time * 2.0;
        }

        if ctx.key_input.down(&Key::D) {
            self.camera.transform.translation +=
                self.camera.transform.local_x() * ctx.delta_time * 2.0;
        }

        if ctx.key_input.pressed(&Key::Left) {
            self.num -= 1;
        }

        if ctx.key_input.pressed(&Key::Right) {
            self.num += 1;
        }

        if ctx.key_input.pressed(&Key::Up) {
            self.agent ^= true;
        }

        if ctx.mouse_input.pressed(&MouseButton::Left) {
            ctx.window.cursor_visible = false;
            ctx.window.cursor_grab = true;
        }

        if ctx.key_input.pressed(&Key::Escape) {
            ctx.window.cursor_visible = true;
            ctx.window.cursor_grab = false;
        }
    }

    #[inline]
    fn render(&mut self, ctx: &mut UpdateCtx) {
        let mut scene = self.scene.pose();
        scene.animate(0, self.time).unwrap();

        let num_objects = (self.num * 2 + 1).pow(2) as usize;

        let mut instances = Vec::with_capacity(num_objects);

        for x in -self.num..=self.num {
            for z in -self.num..=self.num {
                let y = (x as f32 + z as f32 * 0.3 + self.time).sin() * 4.0;
                let mut transform = Transform3d::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0);

                if self.agent {
                    transform.scale = Vec3::splat(0.1);

                    instances.push(transform.matrix());
                } else {
                    transform.translation.y = y;
                    ctx.draw(&self.mesh.transform_material(&transform, &self.metal));
                }
            }
        }

        ctx.draw(&scene.instanced(&instances));

        ctx.draw(&DirectionalLight {
            direction: Vec3::new(-1.0, -1.0, -1.0),
            ..Default::default()
        });

        let fps_transform = Transform3d::from_xyz(0.0, 1.2, 0.0);

        let mut text = TextSprite::new(&self.font, fps_transform);
        text.text = format!("{:2.2} fps | {} objects", 1.0 / ctx.delta_time, num_objects,).into();
        text.size = 0.5;

        if !self.agent {
            ctx.draw(&text);
        }

        ctx.draw(&SkyTexture::new(&self.env));

        ctx.draw(&self.ground);

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
