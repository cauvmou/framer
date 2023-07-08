use framer::Application;
use winit::error::OsError;

#[async_std::main]
pub async fn main() -> Result<(), OsError> {
    let application = Application {
        ..Default::default()
    };

    application.launch().await
}
