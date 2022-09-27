mod renderer;

use renderer::renderer::State;
use winit::{
    event::*,
    event_loop::{ ControlFlow, EventLoop },
    window::{WindowBuilder, Window},
};

pub fn window_setup() -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    (event_loop, window)
}

pub async fn run() {
    env_logger::init();

    let (event_loop, window) = window_setup();

    let mut state = State::new(&window).await;
    
    event_loop.run(
        move | event, _, control_flow | match event {
            Event::NewEvents(_) => {},
            Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    if !state.input(&event) {
                        match event {
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            },
                            WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            },
                            WindowEvent::Resized(physical_size) => {
                                state.resize(physical_size);
                            },
                            WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
                                state.resize(*new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
            },
            Event::DeviceEvent { device_id, event } => {
                
            },
            Event::UserEvent(_) => {},
            Event::Suspended => {},
            Event::Resumed => {},
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.size);
                    },
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                    },
                    Err(e) => {
                        eprintln!("{:?}", e);
                    },
                }
            },
            Event::RedrawEventsCleared => {},
            Event::LoopDestroyed => {},
        }
    );

}

pub fn main() {
    pollster::block_on(run());
}