use glam::Vec2;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    input::Mouse,
    prelude::App,
    renderer::{set_render_ctx, RenderCtx, RenderSurface},
    state::{StartCtx, State, UpdateCtx},
    view::Views,
    window::Window,
};

async unsafe fn wgpu_init(
    window: &winit::window::Window,
) -> anyhow::Result<(RenderCtx, RenderSurface)> {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

    let surface = unsafe { instance.create_surface(window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            force_fallback_adapter: false,
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

    Ok((
        RenderCtx {
            device: ike_wgpu::Device::new(device),
            queue: ike_wgpu::Queue::new(queue),
        },
        RenderSurface::new(ike_wgpu::Surface::new(surface), config),
    ))
}

impl<S: State> App<S> {
    #[inline]
    pub fn run(self, mut state: S) -> ! {
        let event_loop = EventLoop::new();
        let winit_window = WindowBuilder::new().build(&event_loop).unwrap();

        let (render_ctx, mut surface) =
            pollster::block_on(unsafe { wgpu_init(&winit_window) }).unwrap();

        set_render_ctx(render_ctx);

        let mut key_input = Default::default();
        let mut mouse_input = Default::default();
        let mut mouse = Mouse::default();
        let mut char_input = Vec::new();
        let mut window = Window::default();

        window.pre_update(&winit_window);

        let mut start_ctx = StartCtx {
            window: &mut window,
        };

        state.start(&mut start_ctx);

        window.post_update(&winit_window);

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();

                let delta_time = (now - last_frame).as_secs_f32();
                last_frame = now;

                let target = match surface.surface().get_current_texture() {
                    Ok(target) => target,
                    Err(ike_wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("ran out of gpu memory");

                        *control_flow = ControlFlow::Exit;

                        return;
                    }
                    Err(e) => panic!("{:?}", e),
                };

                let target_view = target.texture().create_view(&Default::default());

                let mut views = Views {
                    target: Some(target_view),
                    width: surface.config().width,
                    height: surface.config().height,
                    format: surface.config().format,
                    target_id: None,
                    views: Default::default(),
                };

                window.pre_update(&winit_window);
                let mouse_pos = mouse.position;

                let mut update_ctx = UpdateCtx {
                    delta_time,
                    window: &mut window,
                    key_input: &key_input,
                    mouse_input: &mouse_input,
                    mouse: &mut mouse,
                    char_input: &char_input,
                    views: &mut views,
                };

                state.update(&mut update_ctx);
                state.render(&mut update_ctx);

                window.post_update(&winit_window);

                if mouse.contained && mouse.position != mouse_pos {
                    let _ = winit_window.set_cursor_position(PhysicalPosition::new(
                        mouse.position.x,
                        mouse.position.y,
                    ));
                }

                key_input.update();
                mouse_input.update();
                char_input.clear();
                mouse.update();

                for view in views.views.values() {
                    /*
                    self.renderer
                        .render_view(&render_ctx, delta_time, view, &mut state);
                    */
                }
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
                    surface.configure().width = new_size.width.max(1);
                    surface.configure().height = new_size.height.max(1);
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
                WindowEvent::CursorLeft { .. } => {
                    mouse.contained = false;
                }
                WindowEvent::CursorEntered { .. } => {
                    mouse.contained = true;
                }
                WindowEvent::ReceivedCharacter(c) => {
                    char_input.push(c);
                }
                _ => {}
            },
            _ => {}
        })
    }
}
