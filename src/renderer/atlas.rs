// This unholy garbage should have never seen the light of day.

use image::{DynamicImage, ImageBuffer, Rgba, Rgba32FImage};
use msdf::{GlyphLoader, MSDFConfig, Projection, SDFTrait};
use std::{collections::HashMap, marker::PhantomData};
use texture_packer::{exporter::Exporter, TexturePacker, TexturePackerConfig};
use ttf_parser::GlyphId;

use crate::FONT_RESOURCES;

pub(crate) const SCALE_FACTOR: f64 = 1.0 / 12.0;
pub(crate) const TEXTURE_SCALE: u32 = 2048;

pub struct FontAtlasGenerator {
    msdf_config: MSDFConfig,
    packer_config: TexturePackerConfig,
}

impl FontAtlasGenerator {
    pub fn new() -> Self {
        let msdf_config = msdf::MSDFConfig::default();
        let packer_config = TexturePackerConfig {
            max_width: TEXTURE_SCALE,
            max_height: TEXTURE_SCALE,
            allow_rotation: false,
            border_padding: 0,
            texture_padding: 0,
            ..Default::default()
        };
        Self {
            msdf_config,
            packer_config,
        }
    }

    pub fn with_msdf_config(mut self, config: MSDFConfig) -> Self {
        self.msdf_config = config;
        self
    }

    pub fn generate<'a>(&self, data: Vec<(String, &'a [char])>) -> FontAtlas {
        let mut font_glyphs = Vec::with_capacity(data.len());
        for (name, chars) in data.iter() {
            let face = ttf_parser::Face::from_slice(
                FONT_RESOURCES
                    .read()
                    .unwrap()
                    .get(name)
                    .unwrap_or(&crate::FontResource::default())
                    .bytes,
                0,
            )
            .unwrap();
            let mut glyph_images = Vec::with_capacity(data.len());
            for c in *chars {
                let glyph_index = face.glyph_index(*c).unwrap_or(GlyphId::default());
                let shape = face
                    .load_shape(glyph_index)
                    .unwrap_or(face.load_shape(GlyphId::default()).unwrap());

                let ttf_parser::Rect {
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                } = face
                    .glyph_bounding_box(glyph_index)
                    .unwrap_or(face.glyph_bounding_box(GlyphId::default()).unwrap());
                let width = (x_max - x_min) as f64 * SCALE_FACTOR;
                let height = (y_max - y_min) as f64 * SCALE_FACTOR;

                let glyph_projection = Projection {
                    scale: mint::Vector2 {
                        x: SCALE_FACTOR,
                        y: SCALE_FACTOR,
                    },
                    translation: mint::Vector2 {
                        x: x_min as f64 * -1.0,
                        y: y_min as f64 * -1.0,
                    },
                };

                let colored_shape = shape.color_edges_ink_trap(3.0);
                let mtsdf = colored_shape.generate_mtsdf(
                    width.ceil() as u32,
                    height.ceil() as u32,
                    128.0,
                    &glyph_projection,
                    &self.msdf_config,
                );
                let (hor_advance, ver_advance) = (
                    face.glyph_hor_advance(glyph_index).unwrap_or(0),
                    face.glyph_ver_advance(glyph_index).unwrap_or(0),
                );
                let (hor_side_bearing, ver_side_bearing) = (
                    face.glyph_hor_side_bearing(glyph_index).unwrap_or(0),
                    face.glyph_ver_side_bearing(glyph_index).unwrap_or(0),
                );

                let y_origin = face.glyph_y_origin(glyph_index).unwrap_or(0);
                glyph_images.push((
                    Glyph {
                        name: *c,
                        y_min: y_min.into(),
                        y_max: y_max.into(),
                        x_min: x_min.into(),
                        x_max: x_max.into(),
                        hor_advance,
                        ver_advance,
                        hor_side_bearing,
                        ver_side_bearing,
                        y_origin,
                    },
                    Rgba32FImage::from(mtsdf.to_image()),
                ));
            }
            font_glyphs.push((name.clone(), glyph_images));
        }
        let mut packer = TexturePacker::new_skyline(self.packer_config);
        let glyphs: HashMap<String, Glyph> = font_glyphs.iter().fold(
            HashMap::with_capacity(font_glyphs.len()),
            |mut glyphs, (name, texture)| {
                for (glyph, _) in texture {
                    let mut name = name.to_string();
                    name.push(glyph.name);
                    glyphs.insert(name, *glyph);
                }
                glyphs
            },
        );
        while let Some((name, texture)) = font_glyphs.pop() {
            for (glyph, texture) in texture {
                let mut name = name.to_string();
                name.push(glyph.name);
                packer.pack_own(name, texture).unwrap();
            }
        }
        FontAtlas {
            texture: ImageExporterF32S8::export(&packer).unwrap_or(ImageBuffer::default()),
            packer,
            glyphs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    y_min: f64,
    y_max: f64,
    x_min: f64,
    x_max: f64,
    hor_advance: u16,
    ver_advance: u16,
    hor_side_bearing: i16,
    ver_side_bearing: i16,
    y_origin: i16,
    name: char,
}

impl Glyph {
    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn hor_advance(&self) -> u16 {
        self.hor_advance
    }

    pub fn ver_advance(&self) -> u16 {
        self.ver_advance
    }

    pub fn hor_side_bearing(&self) -> i16 {
        self.hor_side_bearing
    }

    pub fn ver_side_bearing(&self) -> i16 {
        self.ver_side_bearing
    }

    pub fn y_origin(&self) -> i16 {
        self.y_origin
    }
}

pub struct FontAtlas {
    texture: image::RgbaImage,
    packer: TexturePacker<'static, ImageBuffer<Rgba<f32>, Vec<f32>>, String>,
    glyphs: HashMap<String, Glyph>,
}

impl FontAtlas {
    pub fn texture(&self) -> &image::RgbaImage {
        &self.texture
    }

    pub fn glyphs(&self) -> &HashMap<String, Glyph> {
        &self.glyphs
    }

    pub fn packer(&self) -> &TexturePacker<'static, ImageBuffer<Rgba<f32>, Vec<f32>>, String> {
        &self.packer
    }

    pub fn desc<'a>() -> wgpu::TextureDescriptor<'a> {
        wgpu::TextureDescriptor {
            label: Some("Font Atlas"),
            size: wgpu::Extent3d {
                width: TEXTURE_SCALE,
                height: TEXTURE_SCALE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.texture.dimensions()
    }
}

struct ImageExporterF32S8<T: texture_packer::texture::Texture<Pixel = image::Rgba<f32>>> {
    t: PhantomData<T>,
}

impl<T: texture_packer::texture::Texture<Pixel = image::Rgba<f32>>> Exporter<T>
    for ImageExporterF32S8<T>
{
    type Output = image::RgbaImage;

    fn export(texture: &T) -> texture_packer::exporter::ExportResult<Self::Output> {
        let width = texture.width();
        let height = texture.height();

        if width == 0 || height == 0 {
            return Err("Width or height of this texture is zero".to_string());
        }

        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for row in 0..height {
            for col in 0..width {
                if let Some(pixel) = texture.get(col, row) {
                    pixels.push(pixel[0]);
                    pixels.push(pixel[1]);
                    pixels.push(pixel[2]);
                    pixels.push(pixel[3]);
                } else {
                    pixels.push(0.0);
                    pixels.push(0.0);
                    pixels.push(0.0);
                    pixels.push(0.0);
                }
            }
        }

        if let Some(image_buffer) =
            ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(width, height, pixels)
        {
            Ok(DynamicImage::ImageRgba32F(image_buffer).to_rgba8())
        } else {
            Err("Can't export texture".to_string())
        }
    }
}
