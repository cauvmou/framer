use std::sync::Arc;

use winit::{
    dpi::PhysicalSize,
    error::OsError,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub mod font;
mod renderer;

#[derive(Default, Debug, Clone)]
pub struct Application<'a> {
    pub window_config: WindowConfig<'a>,
}

impl<'a> Application<'a> {
    pub async fn launch(self) -> Result<(), OsError> {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop)?;
        window.set_inner_size(PhysicalSize::new(
            self.window_config.size.0,
            self.window_config.size.1,
        ));
        if let Some(min) = self.window_config.min_size {
            window.set_min_inner_size(Some(PhysicalSize::new(min.0, min.1)))
        }
        if let Some(max) = self.window_config.max_size {
            window.set_min_inner_size(Some(PhysicalSize::new(max.0, max.1)))
        }
        window.set_title(self.window_config.title);
        let mut render_state = renderer::State::new(window).await;
        let mut text_state = renderer::text::TextState::new(&render_state);

        text_state.draw(30, 50, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.", Arc::new(font::Font::DEFAULT));
        //text_state.draw(30, 600, "“Hello, World!” gg++-- ÜÜÜ###", Arc::new(font::Font::DEFAULT));
        //text_state.draw(30, 30, "ن بنشوة اللحظة الهائمون في رغباتهم فلا يدركون ما يعقبها من الألم و", Arc::new(font::Font::CAIRO));
        //text_state.draw(30, 600, "\"Hello, World!\" ++--gpq", Arc::new(font::Font::MONOSPACE));

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == render_state.window().id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        render_state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        render_state.resize(**new_inner_size);
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == render_state.window().id() => {
                    match render_state.render(&text_state) {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    render_state.window().request_redraw();
                }
                _ => {}
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowConfig<'a> {
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    max_size: Option<(u32, u32)>,
    title: &'a str,
}

impl<'a> Default for WindowConfig<'a> {
    fn default() -> Self {
        Self {
            size: (800, 600),
            min_size: Some((300, 200)),
            max_size: None,
            title: "Framer-Application",
        }
    }
}
