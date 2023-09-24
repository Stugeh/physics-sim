mod physics;

use physics::{start_physics_thread, PhysicsItem};
use rand::Rng;
use simple_logger::SimpleLogger;
use std::{
    num::NonZeroU32,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    time::Duration,
};
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

type ArcLockPhysxItem = Arc<RwLock<PhysicsItem>>;

pub const CONSTS: PhysicsConsts = PhysicsConsts {
    sand_colors: [
        0b00000000_11000010_10110010_10000000,
        0b00000000_11010010_10101010_01101101,
        0b00000000_11010010_10110111_01101001,
    ],
    gravity: 1,
    update_cycle: Duration::from_millis(50),
};

fn main() {
    SimpleLogger::new().init().unwrap();

    // Color format: 0000 0000 RRRR RRRR GGGG GGGG BBBB BBBB

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
    let physics_objects: Vec<ArcLockPhysxItem> = vec![];
    let physics_objects = Arc::new(RwLock::new(physics_objects));

    let (new_object_sender, new_object_receiver): (
        Sender<ArcLockPhysxItem>,
        Receiver<ArcLockPhysxItem>,
    ) = channel();
    let (redraw_sender, redraw_receiver) = channel();

    let window_dimensions = Arc::new(RwLock::new(WindowDimensions {
        width: window.inner_size().width,
        height: window.inner_size().height,
    }));

    let window_dim_writer = Arc::clone(&window_dimensions);
    let window_dim_reader = Arc::clone(&window_dimensions);

    start_physics_thread(
        window_dim_reader,
        Arc::clone(&physics_objects),
        new_object_receiver,
        redraw_sender,
    );

    let physics_objects_reader = Arc::clone(&physics_objects);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        // If the physics loop has requested for a redraw, redraw
        if let Ok(request) = redraw_receiver.try_recv() {
            if request {
                window.request_redraw();
            }
        }

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
                        // mass: 10,
                        color: CONSTS.sand_colors[rng.gen_range(0..2)],
                    }));

                    new_object_sender
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
                let mut window_dim_writer = window_dim_writer.write().unwrap();
                window_dim_writer.height = window.inner_size().height;
                window_dim_writer.width = window.inner_size().width;

                surface
                    .resize(
                        NonZeroU32::new(window_dim_writer.width).unwrap(),
                        NonZeroU32::new(window_dim_writer.height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();

                buffer.fill(0x00181818);

                let physx_object_reader = physics_objects_reader.read().unwrap();

                for object in physx_object_reader.iter() {
                    let obj = object.read().unwrap();

                    let index = obj.x as usize + obj.y as usize * window_dim_writer.width as usize;

                    if index < buffer.len() {
                        buffer[index] = obj.color;
                    }
                }

                buffer.present().unwrap();
            }
            _ => (),
        };
    })
}

pub struct WindowDimensions {
    width: u32,
    height: u32,
}

pub struct PhysicsConsts {
    pub sand_colors: [u32; 3],
    pub gravity: i32,
    pub update_cycle: Duration,
}
