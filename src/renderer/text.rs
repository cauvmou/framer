use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use image::EncodableLayout;
use ttf_parser::GlyphId;
use wgpu::util::DeviceExt;

use crate::font::{self, Font};

use super::{atlas::FontAtlas, Vertex, Quad};

pub(crate) struct Glyph {
    glyph_id: GlyphId,
    x_advance: f32,
    y_advance: f32,
    x_offset: f32,
    y_offset: f32,
}

pub(crate) struct GlyphSequence {
    glyphs: Vec<Glyph>,
    x: u32,
    y: u32,
    font: Arc<font::Font>,
}

// TODO: Make a global atlas where all font-glyphs are stored in, so that it can be rendered in one drawcall
pub(crate) struct TextState {
    font_to_glyph_ids: HashMap<Arc<Font>, HashSet<GlyphId>>,
    font_to_sequences: HashMap<Arc<Font>, Vec<GlyphSequence>>,
    font_to_atlas: HashMap<Arc<Font>, FontAtlas>,
    pub pipeline: wgpu::RenderPipeline,
}

impl TextState {
    pub fn new(state: &super::State) -> Self {
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

        Self {
            font_to_glyph_ids: HashMap::new(),
            font_to_sequences: HashMap::new(),
            font_to_atlas: HashMap::new(),
            pipeline,
        }
    }

    //TODO: Cache
    pub fn draw(&mut self, x: u32, y: u32, text: &str, font: Arc<font::Font>) {
        let blob = harfbuzz::Blob::new_read_only(font.data);
        let mut buffer = harfbuzz::Buffer::with(text);

        buffer.guess_segment_properties();
        let glyph_sequence = GlyphSequence {
            glyphs: Self::shape(blob, buffer),
            x,
            y,
            font: font.clone(),
        };
        let glyph_ids = {
            if let Some(set) = self.font_to_glyph_ids.get_mut(&font) {
                set
            } else {
                self.font_to_glyph_ids.insert(font.clone(), HashSet::new());
                self.font_to_glyph_ids.get_mut(&font).unwrap()
            }
        };
        let glyph_ids_new = glyph_sequence.glyphs.iter().map(|g| g.glyph_id).collect::<Vec<_>>();
        glyph_ids_new.iter().for_each(|c| {glyph_ids.insert(*c);});
        let atlas = FontAtlas::new(glyph_ids.iter().map(|v|*v).collect::<Vec<_>>(), font.clone()).unwrap();
        atlas.texture.save_with_format("./atlas.png", image::ImageFormat::Png);
        let sequences = {
            if let Some(set) = self.font_to_sequences.get_mut(&font) {
                set
            } else {
                self.font_to_sequences.insert(font.clone(), Vec::new());
                self.font_to_sequences.get_mut(&font).unwrap()
            }
        };
        sequences.push(glyph_sequence);

        self.font_to_atlas.insert(font.clone(), atlas);
    }

    fn shape(blob: harfbuzz::Blob, buffer: harfbuzz::Buffer) -> Vec<Glyph> {
        unsafe {
            harfbuzz::sys::hb_shape(
                harfbuzz::sys::hb_font_create(harfbuzz::sys::hb_face_create(blob.as_raw(), 0)),
                buffer.as_ptr(),
                std::ptr::null(),
                0,
            );
            let mut glyph_count: u32 = 0;
            let glyph_info = harfbuzz::sys::hb_buffer_get_glyph_infos(
                buffer.as_ptr(),
                (&mut glyph_count) as *mut u32,
            );
            let glyph_pos = harfbuzz::sys::hb_buffer_get_glyph_positions(
                buffer.as_ptr(),
                (&mut glyph_count) as *mut u32,
            );
            let glyph_info = &*std::ptr::slice_from_raw_parts_mut(glyph_info, glyph_count as usize);
            let glyph_pos = &*std::ptr::slice_from_raw_parts_mut(glyph_pos, glyph_count as usize);
            (0..glyph_count)
                .map(|i| Glyph {
                    glyph_id: GlyphId(glyph_info[i as usize].codepoint as u16),
                    x_advance: glyph_pos[i as usize].x_advance as f32,
                    y_advance: glyph_pos[i as usize].y_advance as f32,
                    x_offset: glyph_pos[i as usize].x_offset as f32,
                    y_offset: glyph_pos[i as usize].y_offset as f32,
                })
                .collect()
        }
    }

    // TODO: Cache
    pub fn create_texture_binds(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> HashMap<Arc<Font>, wgpu::BindGroup> {
        self.font_to_atlas.iter().map(|(font, atlas)| (font.clone(), Self::create_texture_bind(atlas, device, queue))).collect()
    }

    fn create_texture_bind(atlas: &FontAtlas, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::BindGroup {
        let texture_size = wgpu::Extent3d {
            width: atlas.texture.width(),
            height: atlas.texture.height(),
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
            atlas.texture.as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas.texture.width()), // 4 channels * 4 bytes per channel * number of pixels
                rows_per_image: Some(atlas.texture.height()),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });
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

        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        })
    }

    // TODO: Cache
    pub fn create_buffers(&self, device: &wgpu::Device, screen_width: u32, screen_height: u32) -> HashMap<Arc<Font>, (wgpu::Buffer, wgpu::Buffer, u32)> {
        self.font_to_atlas.iter().map(|(font, atlas)| {
            let glyph_sequences = self.font_to_sequences.get(font).unwrap();
            (font.clone(), Self::create_buffer(font, atlas, glyph_sequences, device, screen_width, screen_height))
        }).collect()
    }

    fn create_buffer(font: &Arc<Font>, atlas: &FontAtlas, glyph_sequences: &Vec<GlyphSequence>, device: &wgpu::Device, screen_width: u32, screen_height: u32) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let scale = 0.3;

        if let Ok(face) = ttf_parser::Face::parse(font.data, 0) {

            for sequence in glyph_sequences {
                let mut cursor = (sequence.x as f32, sequence.y as f32);
                for glyph in &sequence.glyphs {
                    let bound = face.glyph_bounding_box(glyph.glyph_id).unwrap_or(face.glyph_bounding_box(GlyphId::default()).unwrap());
                    let uv = atlas.map.get(&glyph.glyph_id).unwrap();
                    println!("{}", glyph.y_offset);
                    let quad = Quad::new(
                        cursor.0 + glyph.x_offset * scale,
                        cursor.1 + glyph.y_offset * scale,
                        (bound.width()) as f32 * scale,
                        (bound.height()) as f32 * scale,
                        [uv.u, uv.v],
                        [uv.u + uv.w, uv.v + uv.h],
                        screen_width as f32,
                        screen_height as f32,
                    );
                    cursor.0 += glyph.x_advance * scale;
                    cursor.1 += glyph.y_advance * scale;
                    indices.append(&mut quad.indices((vertices.len()) as u16).to_vec());
                    vertices.append(&mut quad.vertices().to_vec());
                }
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
}