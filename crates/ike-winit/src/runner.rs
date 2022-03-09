use ike_app::{App, AppRunner};

use ike_ecs::Events;
use ike_input::{KeyboardInput, MouseButtonInput};
use ike_math::Vec2;
use ike_render::{wgpu, RenderDevice, RenderQueue, Surface};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    convert_mouse_button, convert_virtual_key_code, element_state_pressed, RawWindow, Window,
};

pub struct WinitRunner {
    event_loop: EventLoop<()>,
}

impl WinitRunner {
    pub fn new() -> (Self, Window, Surface, RenderDevice, RenderQueue) {
        let event_loop = EventLoop::new();
        let raw_window = RawWindow::new(&event_loop).unwrap();

        let (window, surface, device, queue) = pollster::block_on(Self::create_window(raw_window));

        (Self { event_loop }, window, surface, device, queue)
    }

    async fn create_window(raw_window: RawWindow) -> (Window, Surface, RenderDevice, RenderQueue) {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        // SAFETY:
        // surface is put into the Window along with raw_window.
        let surface = unsafe { instance.create_surface(&raw_window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("ike_device"),
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let device = RenderDevice::from_raw(device);
        let queue = RenderQueue::from_raw(queue);
        let surface = Surface::from_raw(surface, &device);
        surface.config_surface();
        let window = Window::new(raw_window);

        (window, surface, device, queue)
    }
}

impl AppRunner for WinitRunner {
    fn run(self: Box<Self>, mut app: App) {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::RedrawRequested(_) => {
                    app.update();

                    if let Some(mut window) = app.world.resources_mut().write::<Window>() {
                        window.cursor_delta = Vec2::ZERO;
                    }
                }
                Event::MainEventsCleared => {
                    if let Some(window) = app.world.resources().read::<Window>() {
                        window.request_redraw();
                    }
                }
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta: (x, y) } => {
                        if let Some(mut window) = app.world.resources_mut().write::<Window>() {
                            window.cursor_delta.x += x as f32;
                            window.cursor_delta.y += y as f32;
                        }
                    }
                    _ => {}
                },
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_inner_size) => {
                        if let Some(mut window) = app.world.resources_mut().write::<Surface>() {
                            window.resize(new_inner_size.width, new_inner_size.height);
                        }
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(key),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let input = KeyboardInput {
                            key: convert_virtual_key_code(key),
                            pressed: element_state_pressed(state),
                        };

                        if let Some(mut events) =
                            app.world.resources_mut().write::<Events<KeyboardInput>>()
                        {
                            events.send(input);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let input = MouseButtonInput {
                            button: convert_mouse_button(button),
                            pressed: element_state_pressed(state),
                        };

                        if let Some(mut events) = app
                            .world
                            .resources_mut()
                            .write::<Events<MouseButtonInput>>()
                        {
                            events.send(input);
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if let Some(mut window) = app.world.resources_mut().write::<Window>() {
                            window.cursor_position.x = position.x as f32;
                            window.cursor_position.y = position.y as f32;
                        }
                    }
                    _ => {}
                },
                _ => {}
            });
    }
}
