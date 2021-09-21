use glam::Vec2;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    prelude::App,
    renderer::RenderCtx,
    state::{StartCtx, State, UpdateCtx},
    view::Views,
    window::Window,
};

async unsafe fn wgpu_init(window: &winit::window::Window) -> anyhow::Result<RenderCtx> {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

    let surface = unsafe { instance.create_surface(window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("main device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits {
                    max_texture_dimension_2d: 16_384,
                    ..Default::default()
                },
            },
            None,
        )
        .await?;

    let size = window.inner_size();

    let config = wgpu::SurfaceConfiguration {
        width: size.width,
        height: size.height,
        format: surface.get_preferred_format(&adapter).unwrap(),
        present_mode: wgpu::PresentMode::Fifo,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };

    surface.configure(&device, &config);

    Ok(RenderCtx {
        device: ike_wgpu::Device::new(device).into(),
        queue: ike_wgpu::Queue::new(queue).into(),
        surface: ike_wgpu::Surface::new(surface),
        config,
    })
}

impl<S: State> App<S> {
    #[inline]
    pub fn run(mut self, mut state: S) -> ! {
        let event_loop = EventLoop::new();
        let winit_window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut render_ctx = pollster::block_on(unsafe { wgpu_init(&winit_window) }).unwrap();

        let mut key_input = Default::default();
        let mut mouse_input = Default::default();
        let mut mouse = Default::default();
        let mut char_input = Vec::new();
        let mut window = Window::default();

        window.pre_update(&winit_window);

        let mut start_ctx = StartCtx {
            window: &mut window,
            render_ctx: &render_ctx,
        };

        state.start(&mut start_ctx);

        window.post_update(&winit_window);

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();

                let delta_time = (now - last_frame).as_secs_f32();
                last_frame = now;

                let target = match render_ctx.surface.get_current_frame() {
                    Ok(target) => target,
                    Err(ike_wgpu::SurfaceError::Outdated) => {
                        render_ctx
                            .surface
                            .configure(&render_ctx.device, &render_ctx.config);

                        return;
                    }
                    Err(ike_wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("ran out of gpu memory");

                        *control_flow = ControlFlow::Exit;

                        return;
                    }
                    Err(e) => panic!("{:?}", e),
                };

                let target_view = target.output.create_view(&Default::default());

                let mut views = Views {
                    target: Some(target_view),
                    width: render_ctx.config.width,
                    height: render_ctx.config.height,
                    format: render_ctx.config.format,
                    target_id: None,
                    views: Default::default(),
                };

                window.pre_update(&winit_window);

                let mut update_ctx = UpdateCtx {
                    delta_time,
                    window: &mut window,
                    key_input: &key_input,
                    mouse_input: &mouse_input,
                    mouse: &mouse,
                    char_input: &char_input,
                    render_ctx: &render_ctx,
                    frame: self.renderer.frame(),
                    views: &mut views,
                };

                state.update(&mut update_ctx);
                state.render(&mut update_ctx);

                window.post_update(&winit_window);

                key_input.update();
                mouse_input.update();
                char_input.clear();
                mouse.update();

                for view in views.views.values() {
                    self.renderer.render_view(&render_ctx, view, &mut state);
                }

                self.renderer.clear_nodes();
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (x, y) } => {
                    mouse.movement += Vec2::new(x as f32, y as f32);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                winit_window.request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    if state.exit() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::Resized(new_size) => {
                    render_ctx.config.width = new_size.width.max(1);
                    render_ctx.config.height = new_size.height.max(1);
                    render_ctx
                        .surface
                        .configure(&render_ctx.device, &render_ctx.config);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => match state {
                    ElementState::Pressed => {
                        key_input.press(key);
                    }
                    ElementState::Released => {
                        key_input.release(key);
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => match state {
                    ElementState::Pressed => {
                        mouse_input.press(button);
                    }
                    ElementState::Released => {
                        mouse_input.release(button);
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    mouse.position = Vec2::new(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(x, y) => mouse.wheel_delta += Vec2::new(x, y),
                    MouseScrollDelta::PixelDelta(delta) => {
                        mouse.wheel_delta += Vec2::new(delta.x as f32, delta.y as f32);
                    }
                },
                WindowEvent::ReceivedCharacter(c) => {
                    char_input.push(c);
                }
                _ => {}
            },
            _ => {}
        })
    }
}
