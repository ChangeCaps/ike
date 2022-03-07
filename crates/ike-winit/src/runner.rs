use ike_app::{App, AppRunner};

use ike_render::{wgpu, RenderDevice, RenderQueue, Surface};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{RawWindow, Window};

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
                }
                Event::MainEventsCleared => {
                    app.world.resource::<Window>().request_redraw();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_inner_size) => {
                        app.world
                            .resource_mut::<Surface>()
                            .resize(new_inner_size.width, new_inner_size.height);
                    }
                    _ => {}
                },
                _ => {}
            });
    }
}
