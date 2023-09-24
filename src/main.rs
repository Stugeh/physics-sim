use simple_logger::SimpleLogger;
use std::{
    num::NonZeroU32,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::{self, WindowBuilder},
};

fn main() {
    SimpleLogger::new().init().unwrap();
    const PHYSICS: PhysicsConsts = PhysicsConsts {
        gravity: 1,
        update_cycle: Duration::from_millis(15),
    };

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Simulation")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut cursor_position = LogicalPosition::<i32>::new(0, 0);

    let physics_objects: Arc<RwLock<Vec<Arc<RwLock<PhysicsItem>>>>> = Arc::new(RwLock::new(vec![]));

    let the_matrix = create_arc_vec(None);

    let matrix_width = window.inner_size().width;
    let matrix_height = window.inner_size().height;

    (0..matrix_height).for_each(|_| {
        the_matrix
            .write()
            .unwrap()
            .push(create_empty_arc_vec_with_cap::<PhysicsItem>(
                matrix_width as usize,
            ))
    });

    // Physics update thread
    // let thread_physics_objects = physics_objects.clone();
    // thread::spawn(move || loop {
    //     println!("thread");
    //     let physics_objects = thread_physics_objects.lock().unwrap();
    //     if let Some(object) = physics_objects.last() {
    //         let mut object = object.lock().unwrap();
    //         object.y -= 50;
    //     }
    //     thread::sleep(Duration::from_millis(100));
    // });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        println!("loop");

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    let mut physics_objects = physics_objects.write().unwrap();

                    let new_object = Arc::new(RwLock::new(PhysicsItem {
                        x: cursor_position.x,
                        vx: 0,
                        y: cursor_position.y,
                        vy: 0,
                        mass: 10,
                    }));
                    physics_objects.push(new_object);

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

                let physics_objects = physics_objects.read().unwrap();
                for (index, object) in physics_objects.iter().enumerate() {
                    let mut object = object.write().unwrap();
                    object.vy += PHYSICS.gravity;

                    let buffer_index = object.y as usize * width as usize + object.x as usize;
                    let mut indeces = vec![buffer_index];
                    for i in 1..5 {
                        indeces.push(buffer_index + i);
                        indeces.push(buffer_index + i * width as usize);
                        for j in 1..5 {
                            indeces.push(buffer_index + i * width as usize + j);
                        }
                    }

                    if *indeces.last().unwrap() < buffer.len() {
                        for index in indeces {
                            buffer[index] = u32::MAX
                        }
                    } else {
                        physics_objects.clone().remove(index);
                    }
                }

                buffer.present().unwrap();
            }
            _ => (),
        };

        window.request_redraw();
    })
}

fn create_arc_vec<T>(input_data: Option<T>) -> Arc<RwLock<Vec<T>>> {
    let vec = match input_data {
        Some(data) => vec![data],
        None => vec![],
    };

    Arc::new(RwLock::new(vec))
}

fn create_empty_arc_vec_with_cap<T>(cap: usize) -> Arc<RwLock<Vec<T>>> {
    let vec = Vec::with_capacity(cap);
    Arc::new(RwLock::new(vec))
}

struct PhysicsItem {
    x: i32,
    vx: i32,
    y: i32,
    vy: i32,
    mass: u8,
}

struct PhysicsConsts {
    gravity: i32,
    update_cycle: Duration,
}

enum PhysicsEvent {
    Gravity,
}
