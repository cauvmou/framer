use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
};

use image::ImageBuffer;
use msdf::{GlyphLoader, SDFTrait};
use texture_packer::{
    exporter::{Exporter},
    TexturePacker,
};
use threadpool::ThreadPool;
use ttf_parser::GlyphId;

use crate::{font, renderer::UVRect};



pub(crate) struct FontAtlas {
    pub map: HashMap<GlyphId, UVRect>,
    pub texture: image::RgbaImage,
}

impl FontAtlas {

    pub(crate) const SCALE_FACTOR: f64 = 1.0 / 8.0;
    pub(crate) const TEXTURE_SCALE: u32 = 4096;
    const THREAD_COUNT: usize = 8;

    // TODO: Optimize
    pub fn new(
        ids: Vec<GlyphId>,
        font: Arc<font::Font>,
    ) -> Result<Self, ttf_parser::FaceParsingError> {
        let pool = ThreadPool::new(Self::THREAD_COUNT);
        let msdf_config = Arc::new(msdf::MSDFConfig {
            ..Default::default()
        });
        let packer_config = texture_packer::TexturePackerConfig {
            max_width: Self::TEXTURE_SCALE,
            max_height: Self::TEXTURE_SCALE,
            allow_rotation: false,
            border_padding: 0,
            texture_padding: 0,
            ..Default::default()
        };
    
        let face = Arc::new(ttf_parser::Face::parse(font.data, 0)?);

        let (tx, rx) = mpsc::channel();
        
        for glyph_id in ids {
            let face = face.clone();
            let msdf_config = msdf_config.clone();
            let tx = tx.clone();
            pool.execute(move || {
                let shape = face.load_shape(glyph_id).unwrap_or(face.load_shape(GlyphId::default()).unwrap());
                    let shape = shape.color_edges_ink_trap(3.0);
                    let ttf_parser::Rect {
                        x_min,
                        y_min,
                        x_max,
                        y_max,
                    } = face.glyph_bounding_box(glyph_id).unwrap_or(
                        face.glyph_bounding_box(ttf_parser::GlyphId::default())
                            .expect("THIS IS THE POINT OF DEATH"),
                    );
                    let glyph_projection = msdf::Projection {
                        scale: mint::Vector2 {
                            x: Self::SCALE_FACTOR,
                            y: Self::SCALE_FACTOR,
                        },
                        translation: mint::Vector2 {
                            x: x_min as f64 * -1.0,
                            y: y_min as f64 * -1.0,
                        },
                    };
                    let img = shape
                        .generate_msdf(
                            ((x_max - x_min) as f64 * Self::SCALE_FACTOR).ceil() as u32,
                            ((y_max - y_min) as f64 * Self::SCALE_FACTOR).ceil() as u32,
                            64.0,
                            &glyph_projection,
                            &msdf_config,
                        )
                        .to_image();
    
                    tx.send((glyph_id, img)).expect(":.(");
            });
        }
        pool.join();
        drop(tx);

        let images = rx.iter().collect::<Vec<_>>();
        let mut packer = TexturePacker::new_skyline(packer_config);
        for (key, texture) in images {
            match packer.pack_own(key, texture) {
                Err(err) => eprintln!(
                    "Texture too large for atlas.\nErr -> {err:?}"
                ),
                _ => (),
            }
        }
        
        let texture = ImageExporterF32S8::export(&packer).unwrap_or(image::ImageBuffer::default());
    
        let map: HashMap<GlyphId, UVRect> = packer
            .get_frames()
            .iter()
            .map(|(key, val)| {
                (key.clone(), {
                    let texture_packer::Rect { x, y, w, h } = val.frame;
    
                    UVRect {
                        u: x as f32 / texture.width() as f32,
                        v: y as f32 / texture.height() as f32,
                        w: w as f32 / texture.width() as f32,
                        h: h as f32 / texture.height() as f32,
                    }
                })
            })
            .collect();
        Ok(Self {
            map,
            texture,
        })
    }

    pub fn desc<'a>() -> wgpu::TextureDescriptor<'a> {
        wgpu::TextureDescriptor {
            label: Some("Font Atlas"),
            size: wgpu::Extent3d {
                width: Self::TEXTURE_SCALE,
                height: Self::TEXTURE_SCALE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }
    }
}



struct ImageExporterF32S8A<T: texture_packer::texture::Texture<Pixel = image::Rgba<f32>>> {
    t: std::marker::PhantomData<T>,
}

 
impl<T: texture_packer::texture::Texture<Pixel = image::Rgba<f32>>>
    texture_packer::exporter::Exporter<T> for ImageExporterF32S8A<T>
{
    type Output = image::RgbaImage;

    fn export(texture: &T) -> texture_packer::exporter::ExportResult<Self::Output> {
        let width = texture.width();
        let height = texture.height();

        if width == 0 || height == 0 {
            return Err("Width or height of this texture is zero".to_string());
        }

        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        // TODO: Optimize, this shit is slow af but this fucking texture packer lib seems not to support any other methods. I present the Uga-Buga-Copy method
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
            image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::from_raw(width, height, pixels)
        {
            Ok(image::DynamicImage::ImageRgba32F(image_buffer).to_rgba8())
        } else {
            Err("Can't export texture".to_string())
        }
    }
}

struct ImageExporterF32S8<T: texture_packer::texture::Texture<Pixel = image::Rgb<f32>>> {
    t: std::marker::PhantomData<T>,
}

 
impl<T: texture_packer::texture::Texture<Pixel = image::Rgb<f32>>>
    texture_packer::exporter::Exporter<T> for ImageExporterF32S8<T>
{
    type Output = image::RgbaImage;

    fn export(texture: &T) -> texture_packer::exporter::ExportResult<Self::Output> {
        let width = texture.width();
        let height = texture.height();

        if width == 0 || height == 0 {
            return Err("Width or height of this texture is zero".to_string());
        }

        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        // TODO: Optimize, this shit is slow af but this fucking texture packer lib seems not to support any other methods. I present the Uga-Buga-Copy method
        for row in 0..height {
            for col in 0..width {
                if let Some(pixel) = texture.get(col, row) {
                    pixels.push(pixel[0]);
                    pixels.push(pixel[1]);
                    pixels.push(pixel[2]);
                    pixels.push(1.0);
                } else {
                    pixels.push(0.0);
                    pixels.push(0.0);
                    pixels.push(0.0);
                    pixels.push(1.0);
                }
            }
        }

        if let Some(image_buffer) =
            image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::from_raw(width, height, pixels)
        {
            Ok(image::DynamicImage::ImageRgba32F(image_buffer).to_rgba8())
        } else {
            Err("Can't export texture".to_string())
        }
    }
}