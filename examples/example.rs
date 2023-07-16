use framer::{error::FramerError, FramerApplication, FramerConfig};

#[async_std::main]
pub async fn main() -> Result<(), FramerError> {
    FramerApplication::new(&FramerConfig {
        ..Default::default()
    })
    .launch(move || {})
}
