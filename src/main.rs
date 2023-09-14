use std::{collections::HashMap, num::NonZeroU32};

use simple_logger::SimpleLogger;
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("tterm")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut cursor_position = 0;

    event_loop.run(move |event, _, control_flow| {
        // Run loop only when there are events happening
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    device_id: _,
                    modifiers: _,
                } => handle_mouse(cursor_position),
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

                buffer.present().unwrap();
            }
            _ => (),
        };
    })
}

fn handle_mouse(cursor_position: u32) {
    println!("handling mouse")
}
