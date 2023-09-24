use std::{
    collections::HashSet,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use crate::{WindowDimensions, CONSTS};
pub type ArcLockPhysxItem = Arc<RwLock<PhysicsItem>>;

pub fn start_physics_thread(
    window_dim_reader: Arc<RwLock<WindowDimensions>>,
    physics_object_writer: Arc<RwLock<Vec<ArcLockPhysxItem>>>,
    new_object_receiver: Receiver<ArcLockPhysxItem>,
    redraw_sender: Sender<bool>,
) {
    thread::spawn(move || loop {
        thread::sleep(CONSTS.update_cycle);
        let mut physics_object_writer = physics_object_writer.write().unwrap();

        for new_object in new_object_receiver.try_iter() {
            physics_object_writer.push(new_object);
        }

        // We need to update item positions from the bottom up so we need to keep the array of
        // PhysicsItems sorted by their coordinates. Insertion sort is fast on partially sorted
        // Vecs.
        for i in 1..physics_object_writer.len() {
            let mut j = i;
            while j > 0
                && physics_object_writer[j].read().unwrap().y
                    > physics_object_writer[j - 1].read().unwrap().y
                && physics_object_writer[j].read().unwrap().x
                    > physics_object_writer[j - 1].read().unwrap().x
            {
                physics_object_writer.swap(j, j - 1);
                j -= 1;
            }
        }

        let window_dim_reader = window_dim_reader.read().unwrap();

        let mut occupied_positions = HashSet::new();
        // update position and velocity of each object
        for object in physics_object_writer.iter() {
            let mut current_obj = object.write().unwrap();

            // If object hasnt reached the bottom of the screen update y and apply gravity
            if current_obj.y + current_obj.vy < window_dim_reader.height as i32 {
                current_obj.y += current_obj.vy;
                current_obj.vy += CONSTS.gravity;

                // Check for collision
                let mut new_position = (current_obj.x, current_obj.y);
                while occupied_positions.contains(&new_position) {
                    new_position.1 -= 1
                }
                current_obj.y = new_position.1;

                continue;
            }

            current_obj.x += current_obj.vx;
            current_obj.y = (window_dim_reader.height - 5) as i32;

            let mut new_position = (current_obj.x, current_obj.y);
            while occupied_positions.contains(&new_position) {
                new_position.1 -= 1
            }
            current_obj.y = new_position.1;
            occupied_positions.insert(new_position);
        }

        // send redraw request
        redraw_sender
            .clone()
            .send(true)
            .expect("Failed to ask for redraw");
    });
}

pub struct PhysicsItem {
    pub x: i32,
    pub vx: i32,
    pub y: i32,
    pub vy: i32,
    pub color: u32,
    // mass: u8,
}
