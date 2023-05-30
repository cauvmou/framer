use framer::renderer::atlas::FontAtlasGenerator;

fn main() {
    let chars = &(32..=126)
        .map(|value| value as u8 as char)
        .collect::<Vec<char>>();
    let generator = FontAtlasGenerator::new();
    let atlas = generator.generate(vec![("Roboto-Regular".to_string(), &chars)]);
    {
        let texture = atlas.texture().clone();
        let dynamic: image::DynamicImage = texture.into();
        dynamic.to_rgb8().save("./generated/atlas.png").unwrap();
    }
}
