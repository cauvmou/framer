use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    renderer::{text::TextState, State},
    Text,
};

pub struct Window {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
}

impl Window {
    pub fn new(config: &Config) -> Self {
        let event_loop = EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_decorations(config.decorations)
            .with_resizable(config.resizable)
            .with_maximized(config.maximized)
            .with_inner_size(Size::Physical(PhysicalSize::from(config.size)))
            .with_title(config.title)
            .build(&event_loop)
            .unwrap();
        Self { event_loop, window }
    }

    pub async fn launch(self, text: Text) {
        let Self { event_loop, window } = self;

        let mut render_state = State::new(window).await;
        let mut text_state = TextState::new(&render_state);
        text_state.push(text.clone());

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
                    match render_state.render(&mut text_state) {
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
        });
    }
}

pub struct Config<'a> {
    pub resizable: bool,
    pub maximized: bool,
    pub decorations: bool,
    pub size: (u32, u32),
    pub title: &'a str,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            resizable: true,
            maximized: false,
            decorations: true,
            size: (800, 600),
            title: "Framer Application",
        }
    }
}
