use std::sync::Arc;

use threadpool::ThreadPool;

use crate::font;

pub(crate) const SCALE_FACTOR: f64 = 1.0 / 8.0;
pub(crate) const TEXTURE_SCALE: u32 = 4096;
const THREAD_COUNT: usize = 4;

pub(crate) fn generate_atlas(codepoints: Vec<u32>, font: Arc<font::Font>) {
    let pool = ThreadPool::new(THREAD_COUNT);
    let msdf_config = msdf::MSDFConfig::default();
    let packer_config = texture_packer::TexturePackerConfig {
        max_width: TEXTURE_SCALE,
        max_height: TEXTURE_SCALE,
        allow_rotation: false,
        border_padding: 0,
        texture_padding: 0,
        ..Default::default()
    };

    for codepoint in codepoints {
        pool.execute(move || {
            
        });
    }

    todo!()
}