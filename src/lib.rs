use error::FramerError;

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod element;
pub mod error;
pub mod layout;
pub mod reactive;
pub mod render;
pub mod style;
pub(crate) mod util;

pub struct FramerApplication {}

impl FramerApplication {
    pub fn new(_config: &FramerConfig) -> Self {
        Self {}
    }

    pub fn launch<F, R>(self, _mount: F) -> Result<(), FramerError>
    where
        F: FnOnce() -> R + 'static,
        R: std::any::Any,
    {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_decorations(false)
            .build(&event_loop)?;

        let mut cursor_in_window = false;
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                winit::event::Event::WindowEvent { window_id, event }
                    if window_id == window.id() =>
                {
                    match event {
                        winit::event::WindowEvent::ReceivedCharacter(c) => {
                            println!("GOT: {c}");
                        }
                        winit::event::WindowEvent::CursorEntered { device_id: _ } => {
                            cursor_in_window = true
                        }
                        winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                            cursor_in_window = false
                        }
                        winit::event::WindowEvent::MouseInput {
                            device_id: _,
                            state: _,
                            button,
                            modifiers: _,
                        } => match button {
                            winit::event::MouseButton::Left => {
                                if cursor_in_window {
                                    let _ = window.drag_window();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                winit::event::Event::MainEventsCleared => {
                    //
                }
                winit::event::Event::RedrawRequested(window_id) if window_id == window.id() => {}
                _ => {}
            }
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FramerConfig {
    pub width: u64,
    pub height: u64,
    pub min_width: u64,
    pub min_height: u64,
    pub max_width: u64,
    pub max_height: u64,
    pub resizable: bool,
}

impl Default for FramerConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            min_width: 320,
            min_height: 240,
            max_width: u64::MAX,
            max_height: u64::MAX,
            resizable: true,
        }
    }
}
