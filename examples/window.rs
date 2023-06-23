use framer::{
    window::{Config, Window},
    Text,
};

#[async_std::main]
async fn main() {
    let window = Window::new(&Config {
        ..Default::default()
    });

    window.launch(Text::new("AWgx")).await;
}
