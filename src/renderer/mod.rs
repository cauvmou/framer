use winit::window::Window;

use self::text::TextState;

pub(crate) mod atlas;
pub(crate) mod text;

const BACKENDS: Option<wgpu::Backends> = wgpu::Backends::from_bits(
    wgpu::Backends::VULKAN.bits() | wgpu::Backends::GL.bits() | wgpu::Backends::METAL.bits(),
);

pub(crate) struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    color_buffer: wgpu::Texture,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS.unwrap_or(wgpu::Backends::GL),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
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

        let color_buffer = Self::create_color_buffer(&device, size.width as f64, size.height as f64, 1.0, 4, surface_format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            color_buffer,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.color_buffer.destroy();
            self.color_buffer = Self::create_color_buffer(&self.device, new_size.width as f64, new_size.height as f64, 1.0, 4, self.config.format);
        }
    }

    pub fn render(
        &mut self,
        text_state: &TextState,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let color_view = &self.color_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let textures = text_state.create_texture_binds(&self.device, &self.queue);
        let buffers = text_state.create_buffers(&self.device, self.size.width, self.size.height);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&text_state.pipeline);
            for font in textures.keys().into_iter() {
                render_pass.set_bind_group(0, textures.get(font).unwrap(), &[]);
                let (vertex_buffer, index_buffer, num_indices) = buffers.get(font).unwrap();
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn create_color_buffer(
        device: &wgpu::Device,
        width: f64,
        height: f64,
        pixel_ratio: f64,
        sample_count: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: (width * pixel_ratio) as u32,
                height: (height * pixel_ratio) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            view_formats: &[],
        })
    }
}

pub struct UVRect {
    u: f32,
    v: f32,
    w: f32,
    h: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// TODO: Hölle
#[derive(Debug, Clone, Copy)]
struct Quad {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    uv: [f32; 2],
    xy: [f32; 2],
}

impl Quad {
    pub fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv: [f32; 2],
        xy: [f32; 2],
        sw: f32,
        sh: f32,
    ) -> Self {
        Self {
            x1: (2.0 * x / sw) - 1.0,
            y1: (2.0 * y / sh) - 1.0,
            x2: (2.0 * (x + w) / sw) - 1.0,
            y2: (2.0 * (y + h) / sh) - 1.0,
            uv,
            xy,
        }
    }

    pub fn vertices(&self) -> [Vertex; 4] {
        [
            Vertex {
                position: [self.x1, self.y1],
                uv: [self.uv[0], self.xy[1]],
            },
            Vertex {
                position: [self.x2, self.y1],
                uv: [self.xy[0], self.xy[1]],
            },
            Vertex {
                position: [self.x2, self.y2],
                uv: [self.xy[0], self.uv[1]],
            },
            Vertex {
                position: [self.x1, self.y2],
                uv: [self.uv[0], self.uv[1]],
            },
        ]
    }

    pub fn indices(&self, starting: u16) -> [u16; 6] {
        [
            starting + 0,
            starting + 1,
            starting + 2,
            starting + 0,
            starting + 2,
            starting + 3,
        ]
    }
}