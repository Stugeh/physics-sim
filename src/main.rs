use std::{collections::HashMap, num::NonZeroU32};

use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::{self, Window, WindowBuilder},
};

fn main() {
    SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Simulation")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut cursor_position = LogicalPosition::<i32>::new(0, 0);
    let mut add_new_entity = false;

    event_loop.run(move |event, _, control_flow| {
        // Run loop only when there are events happening
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    add_new_entity = true;
                    window.request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let pos = position.to_logical(1.0);
                    cursor_position.x = pos.x;
                    cursor_position.y = pos.y;
                }
                _ => {}
            },

            Event::RedrawRequested(_) => {
                //notify windowing system that we'll be presenting to the window

                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();
                let mut buffer = surface.buffer_mut().unwrap();

                buffer.fill(0x00181818);

                if add_new_entity {
                    //calculate pixel the cursor is on
                    let pixel = window.inner_size().width * cursor_position.y as u32
                        + cursor_position.x as u32;

                    buffer[pixel as usize] = u32::MAX;
                }

                buffer.present().unwrap();
            }
            _ => (),
        };
    })
}
