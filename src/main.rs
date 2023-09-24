use rand::Rng;
use simple_logger::SimpleLogger;
use std::{
    num::NonZeroU32,
    sync::{mpsc::channel, Arc, RwLock},
    thread,
    time::Duration,
};
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();

    // Color format: 0000 0000 RRRR RRRR GGGG GGGG BBBB BBBB
    const CONSTS: PhysicsConsts = PhysicsConsts {
        sand_colors: [
            0b00000000_11000010_10110010_10000000,
            0b00000000_11010010_10101010_01101101,
            0b00000000_11010010_10110111_01101001,
        ],
        gravity: 1,
        update_cycle: Duration::from_millis(100),
    };

    // Init winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Simulation")
        .build(&event_loop)
        .unwrap();

    // Init drawing lib
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let mut cursor_position = LogicalPosition::<i32>::new(0, 0);

    // init physics_objects vector
    let physics_objects: Vec<Arc<RwLock<PhysicsItem>>> = vec![];
    let physics_objects = Arc::new(RwLock::new(physics_objects));

    let (sender, receiver) = std::sync::mpsc::channel();

    // Thread where all updates to the physics objects vector should be handled
    let mut thread_physics_objects = physics_objects.clone().write().unwrap().clone();
    thread::spawn(move || loop {
        thread::sleep(CONSTS.update_cycle);

        for new_object in receiver.try_iter() {
            thread_physics_objects.push(new_object);
        }

        // We need to update item positions from the bottom up so we need to keep the array of
        // PhysicsItems sorted by their y coordinate. Insertion sort is fast on partially sorted
        // Vecs.
        for i in 1..thread_physics_objects.len() {
            let mut j = i;
            while j > 0
                && thread_physics_objects[j].read().unwrap().y
                    < thread_physics_objects[j - 1].read().unwrap().y
            {
                thread_physics_objects.swap(j, j - 1);
                j -= 1;
            }
        }

        // update position and velocity of each object
        for object in thread_physics_objects.iter().rev() {
            let mut current_obj = object.write().unwrap();
            current_obj.y += current_obj.vy;
            current_obj.vy += CONSTS.gravity;
            current_obj.x += current_obj.vx;
        }
    });

    let physx_object_reader = physics_objects.read().unwrap().clone();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    let mut rng = rand::thread_rng();

                    let new_object = Arc::new(RwLock::new(PhysicsItem {
                        x: cursor_position.x,
                        vx: 0,
                        y: cursor_position.y,
                        vy: 0,
                        mass: 10,
                        color: CONSTS.sand_colors[rng.gen_range(0..2)],
                    }));

                    sender
                        .send(new_object)
                        .expect("this should always succeed since receiver is never killed");
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let pos = position.to_logical(1.0);
                    cursor_position.x = pos.x;
                    cursor_position.y = pos.y;
                }
                _ => {}
            },

            Event::RedrawRequested(_) => {
                let (window_width, window_height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                surface
                    .resize(
                        NonZeroU32::new(window_width).unwrap(),
                        NonZeroU32::new(window_height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();

                buffer.fill(0x00181818);

                for object in physx_object_reader.iter() {
                    let obj = object.read().unwrap();

                    let index = obj.x as usize + obj.y as usize * window_width as usize;

                    if index < buffer.len() {
                        buffer[index] = object.read().unwrap().color;
                    }
                }

                buffer.present().unwrap();
            }
            _ => (),
        };

        window.request_redraw();
        thread::sleep(CONSTS.update_cycle);
    })
}

struct PhysicsItem {
    x: i32,
    vx: i32,
    y: i32,
    vy: i32,
    color: u32,
    mass: u8,
}

struct PhysicsConsts {
    sand_colors: [u32; 3],
    gravity: i32,
    update_cycle: Duration,
}
