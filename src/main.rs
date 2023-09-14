use simple_logger::SimpleLogger;
use std::{num::NonZeroU32, sync::mpsc, thread, time::Duration};
use uuid::Uuid;
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();
    const _PHYSICS: PhysicsConsts = PhysicsConsts {
        gravity: 1,
        update_cycle: Duration::from_millis(10),
    };

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Simulation")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut cursor_position = LogicalPosition::<i32>::new(0, 0);

    // let (tx, _rx) = mpsc::channel();
    let mut physics_objects: Vec<PhysicsItem> = vec![];
    // let mut handles = vec![];

    // Physics update thread
    // for object in physics_objects {
    //     let handle = thread::spawn(move || loop {
    //         tx.send((
    //             270,
    //             object.velocity_vector.velocity + PHYSICS.gravity,
    //             object.velocity_vector.velocity,
    //         ));
    //     });
    //     handles.push(handle);
    //     thread::sleep(PHYSICS.update_cycle);
    // }

    event_loop.run(move |event, _, control_flow| {
        // Run loop only when there are events happening
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    physics_objects.push(PhysicsItem {
                        item_id: Uuid::new_v4(),
                        velocity_vector: VelocityVector {
                            direction: 0,
                            velocity: 0,
                        },
                        has_gravity: true,
                        x: cursor_position.x,
                        y: cursor_position.y,
                        mass: 10,
                    });
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

                physics_objects.iter().for_each(|item| {
                    buffer[item.y as usize * width as usize + item.x as usize] = u32::MAX
                });

                buffer.present().unwrap();
            }
            _ => (),
        };
    })
}

struct PhysicsItem {
    item_id: Uuid,
    velocity_vector: VelocityVector,
    has_gravity: bool,
    y: i32,
    x: i32,
    mass: u8,
}

struct PhysicsConsts {
    gravity: u8,
    update_cycle: Duration,
}

struct VelocityVector {
    direction: u16,
    velocity: u8,
}
