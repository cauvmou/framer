use wgpu::util::DeviceExt;

use crate::Text;

use super::{
    atlas::{FontAtlas, FontAtlasGenerator},
    State,
};

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

#[derive(Debug, Clone, Copy)]
struct Quad {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    xy: [f32; 2],
    uv: [f32; 2],
}

impl Quad {
    pub fn new(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        xy: [f32; 2],
        uv: [f32; 2],
        sw: f32,
        sh: f32,
    ) -> Self {
        Self {
            x1: (2.0 * x / sw) - 1.0,
            y1: (2.0 * y / sh) - 1.0,
            x2: (2.0 * (x + w) / sw) - 1.0,
            y2: (2.0 * (y + h) / sh) - 1.0,
            xy,
            uv,
        }
    }

    pub fn vertices(&self) -> [Vertex; 4] {
        [
            Vertex {
                position: [self.x1, self.y1],
                uv: [self.xy[0], self.uv[1]],
            },
            Vertex {
                position: [self.x2, self.y1],
                uv: [self.uv[0], self.uv[1]],
            },
            Vertex {
                position: [self.x2, self.y2],
                uv: [self.uv[0], self.xy[1]],
            },
            Vertex {
                position: [self.x1, self.y2],
                uv: [self.xy[0], self.xy[1]],
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

pub(crate) struct TextState {
    atlas: FontAtlas,
    texts: Vec<Text>,
    pipeline: wgpu::RenderPipeline,
    pub(crate) texture_bind: Option<wgpu::BindGroup>,
}

impl TextState {
    pub fn new(state: &State) -> Self {
        // create the pipeline layout
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Text Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/font.wgsl").into()),
            });

        let texture_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("text_texture_bind_group_layout"),
                });

        let pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Text Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Text Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: true,
                },
                multiview: None,
            });

        let atlas = FontAtlasGenerator::new().generate(vec![]);

        Self {
            atlas,
            texts: Vec::new(),
            pipeline,
            texture_bind: None,
        }
    }

    // Lord have mercy
    pub fn push(&mut self, text: Text) {
        self.texts.push(text);
        let data: Vec<(String, Vec<char>)> = self
            .texts
            .iter()
            .map(|t| (t.font.to_owned(), t.literal.chars().collect::<Vec<char>>()))
            .collect();
        self.atlas = FontAtlasGenerator::new().generate(
            data.iter()
                .map(|data| (data.0.to_owned(), data.1.as_slice()))
                .collect(),
        );
        self.texture_bind = None;
    }

    // Returns the vertex buffer, index buffer, and number of indices
    pub fn create_buffers(
        &mut self,
        device: &wgpu::Device,
        screen_width: f32,
        screen_height: f32,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        for text in &self.texts {
            let mut start = (30.0, 80.0);
            let size = 0.05f32;
            let mut prev: char = ' ';
            for c in text.literal.chars() {
                let mut font = text.font.clone();
                font.push(c);
                let glyph = self.atlas.glyphs().get(&font).unwrap();

                let frame = self.atlas.packer().get_frame(&font).unwrap();
                let rect = frame.frame;

                let kerning = glyph.kerning_table.get(&prev).unwrap_or(&0);

                // TODO: REWORK I should just read and then implement not the other way around (https://simoncozens.github.io/fonts-and-layout/concepts.html)
                let quad = Quad::new(
                    start.0 + ((glyph.hor_side_bearing + kerning) as f32 - glyph.x_min as f32) * size,
                    start.1 + ((glyph.y_origin + glyph.ver_side_bearing) as f32 + glyph.y_min as f32) * size,
                    glyph.width() as f32 * size,
                    glyph.height() as f32 * size,
                    [
                        rect.x as f32 / self.atlas.dimensions().0 as f32,
                        rect.y as f32 / self.atlas.dimensions().1 as f32,
                    ],
                    [
                        (rect.x + rect.w) as f32 / self.atlas.dimensions().0 as f32,
                        (rect.y + rect.h) as f32 / self.atlas.dimensions().1 as f32,
                    ],
                    screen_width,
                    screen_height,
                );
                indices.append(&mut quad.indices((vertices.len()) as u16).to_vec());
                vertices.append(&mut quad.vertices().to_vec());
                start.0 += glyph.hor_advance as f32 * size;
                start.1 += glyph.ver_advance as f32 * size;
                prev = c;
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;
        (vertex_buffer, index_buffer, num_indices)
    }

    // Return the texture bind group
    pub fn create_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let texture_size = wgpu::Extent3d {
            width: self.atlas.dimensions().0,
            height: self.atlas.dimensions().1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("font_texture"),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.atlas.texture(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.atlas.dimensions().0),
                rows_per_image: Some(self.atlas.dimensions().1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("font_texture_bind_group_layout"),
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        self.texture_bind = Some(bind_group);
    }

    pub(crate) fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
