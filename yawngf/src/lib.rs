use getset::Getters;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod error;

use error::InitError;

pub use wgpu::PowerPreference;

#[derive(Getters)]
struct Wgpu {
    surface: Surface,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    #[getset(get)]
    window: Window,
}

pub struct Context {
    event_loop: EventLoop<()>,
    wgpu: Wgpu,
}
impl Context {
    ///power preference is for laptops usually that have integrated low-performane or high performance graphic cards. pick which you prefer
    pub async fn new(
        power_preference: PowerPreference,
        force_fallback: bool,
    ) -> Result<Self, error::InitError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let wgpu = Wgpu::new(window, power_preference, force_fallback).await?;

        Ok(Self { event_loop, wgpu })
    }
    pub fn run(self) {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.wgpu.window().id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                _ => {}
            });
    }
}

impl Wgpu {
    ///
    async fn new(
        window: Window,
        power_preference: PowerPreference,
        force_fallback: bool,
    ) -> Result<Self, InitError> {
        use wgpu::{Backends, Instance, InstanceDescriptor, RequestAdapterOptions};
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }?;
        let options = RequestAdapterOptions {
            power_preference,
            force_fallback_adapter: force_fallback,
            compatible_surface: Some(&surface),
        };
        if let Some(adapter) = instance.request_adapter(&options).await {
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        // WebGL doesn't support all of wgpu's features, so if
                        // we're building for the web we'll have to disable some.
                        limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        label: None,
                    },
                    None, // Trace path
                )
                .await?;
            let surface_caps = surface.get_capabilities(&adapter);
            // Shader code in this tutorial assumes an sRGB surface texture. Using a different
            // one will result all the colors coming out darker. If you want to support non
            // sRGB surfaces, you'll need to account for that when drawing to the frame.
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };
            surface.configure(&device, &config);

            Ok(Self {
                size,
                surface,
                device,
                window,
                queue,
                surface_config: config,
            })
        } else {
            Err(InitError::NoAdapter)
        }
    }
}
