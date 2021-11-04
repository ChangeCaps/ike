use ike_core::*;
use ike_input::{Input, Mouse};
use ike_render::*;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub type Key = VirtualKeyCode;
pub use winit::event::MouseButton;

pub struct WinitRunner;

impl AppRunner for WinitRunner {
    #[inline]
    fn run(&mut self, mut app: App) {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).unwrap();

        let (render_ctx, render_surface) =
            pollster::block_on(unsafe { wgpu_init(&window) }).unwrap();

        let window = crate::Window::from_raw(window);

        app.world_mut().insert_resource(render_surface);
        app.world_mut().insert_resource(window);

        app.world_mut().insert_resource(Input::<Key>::default());
        app.world_mut()
            .insert_resource(Input::<MouseButton>::default());
        app.world_mut().insert_resource(Mouse::default());

        set_render_ctx(render_ctx);

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                app.update_components();
                app.execute_schedule();

                app.world().write_resource::<Input<Key>>().unwrap().update();
                app.world()
                    .write_resource::<Input<MouseButton>>()
                    .unwrap()
                    .update();
                let mut mouse = app.world().write_resource::<Mouse>().unwrap();

                mouse.update();

                let window = app.world().read_resource::<crate::Window>().unwrap();

                window.get_raw().set_cursor_visible(mouse.visible);
                window.get_raw().set_cursor_grab(mouse.grabbed).unwrap();
            }
            Event::MainEventsCleared => {
                let window = app.world().read_resource::<crate::Window>().unwrap();

                window.get_raw().request_redraw();
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                    let mut mouse = app.world_mut().write_resource::<Mouse>().unwrap();

                    mouse.movement.x += dx as f32;
                    mouse.movement.y += dy as f32;
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => {
                    let mut render_surface = app.world().write_resource::<RenderSurface>().unwrap();

                    render_surface.configure().width = size.width;
                    render_surface.configure().height = size.height;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    let mut input = app.world_mut().write_resource::<Input<Key>>().unwrap();

                    match state {
                        ElementState::Pressed => {
                            input.press(key);
                        }
                        ElementState::Released => {
                            input.release(key);
                        }
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let mut input = app
                        .world_mut()
                        .write_resource::<Input<MouseButton>>()
                        .unwrap();

                    match state {
                        ElementState::Pressed => {
                            input.press(button);
                        }
                        ElementState::Released => {
                            input.release(button);
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut mouse = app.world_mut().write_resource::<Mouse>().unwrap();

                    mouse.position.x = position.x as f32;
                    mouse.position.y = position.y as f32;
                }
                _ => {}
            },
            _ => {}
        });
    }
}

async unsafe fn wgpu_init(
    window: &winit::window::Window,
) -> anyhow::Result<(RenderCtx, RenderSurface)> {
    let instance = ::wgpu::Instance::new(wgpu::Backends::PRIMARY);

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
