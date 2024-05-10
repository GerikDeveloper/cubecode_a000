use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::sync::mpsc::channel;
use crate::chunk::SubChunk;
use crate::render::blocks_loader::{AIR_BLOCK_ID, BlocksLoader};
use crate::render::light::light_map::LightMap;
use crate::render::types::{Vec3b, Vec3s, Vec3ub};
use crate::world::World;

struct LightEntry {
    pos: Vec3ub,
    light_level: u8,
}

impl LightEntry {
    pub fn new(pos: &Vec3ub, light_level: u8) -> Self {
        return Self {
            pos: pos.clone(),
            light_level,
        };
    }
}

pub struct LightSolver {
    channel: Cell<u8>,
    add_queue: RefCell<VecDeque<LightEntry>>, //add queue
    rem_queue: RefCell<VecDeque<LightEntry>>, //remove queue
}
impl LightSolver {
    pub fn new(channel: u8) -> Self {
        return Self {
            channel: Cell::from(channel),
            add_queue: RefCell::from(VecDeque::new()),
            rem_queue: RefCell::from(VecDeque::new()),
        };
    }

    pub fn add(&self, world: &World, pos: &Vec3ub, emission: u8) {
        if emission <= 1 {return;} //If light does not spread
        let entry: LightEntry = LightEntry::new(pos, emission);
        self.add_queue.borrow_mut().push_back(entry);
        world.set_light_level(pos, self.channel.get(), emission);
    }

    pub fn add_last(&self, world: &World, pos: &Vec3ub) {
        self.add(world, pos, world.get_light_level(pos, self.channel.get()));
    }

    pub fn remove(&self, world: &World, pos: &Vec3ub) {
        let light_level: u8 = world.get_light_level(pos, self.channel.get());
        if light_level == 0 {return;}
        let entry: LightEntry = LightEntry::new(pos, light_level);
        self.rem_queue.borrow_mut().push_back(entry);
        world.set_light_level(pos, self.channel.get(), 0);
    }

    //TODO rewrite get neighbor pos in other file
    pub fn get_neighbor_pos(pos: &Vec3ub, offset: &Vec3b) -> Option<Vec3ub> {
        let exp_sum: Vec3s = [(offset[0] as i16) + (pos[0] as i16), (offset[1] as i16) + (pos[1] as i16), (offset[2] as i16) + (pos[2] as i16)];
        if exp_sum[0] >= 0x00 && exp_sum[0] <= 0xFF &&
            exp_sum[1] >= 0x00 && exp_sum[1] <= 0xFF &&
            exp_sum[2] >= 0x00 && exp_sum[2] <= 0xFF {
            return Some([exp_sum[0] as u8, exp_sum[1] as u8, exp_sum[2] as u8]);
        }
        return None;
    }

    pub fn solve(&self, world: &World, blocks_loader: &BlocksLoader) {
        //TODO MB RENAME TO SHIFT EVERYWHERE
        const NEIGHBORHOOD: [Vec3b; 6] = [[0, 0, -1], [0, 0, 1], [0, -1, 0], [0, 1, 0], [-1, 0, 0], [1, 0, 0]];
        let mut rem_queue: RefMut<VecDeque<LightEntry>> = self.rem_queue.borrow_mut();
        let mut add_queue: RefMut<VecDeque<LightEntry>> = self.add_queue.borrow_mut();
        while !rem_queue.is_empty() {
            if let Some(entry) = rem_queue.pop_front() {
                for neigh in NEIGHBORHOOD {
                    if let Some(neigh_pos) = Self::get_neighbor_pos(&entry.pos, &neigh) {
                        let light_level = world.get_light_level(&neigh_pos, self.channel.get());
                        let neigh_entry: LightEntry = LightEntry::new(&neigh_pos, light_level);
                        if (light_level != 0) && (light_level == (entry.light_level - 1)) {
                            rem_queue.push_back(neigh_entry);
                            world.set_light_level(&neigh_pos, self.channel.get(), 0);
                        } else if light_level >= entry.light_level {
                            add_queue.push_back(neigh_entry);
                        }
                    }
                }
            } else {
                break;
            }
        }

        while !add_queue.is_empty() {
            if let Some(entry) = add_queue.pop_front() {
                if entry.light_level <= 1 {continue;}
                for neigh in NEIGHBORHOOD {
                    if let Some(neigh_pos) = Self::get_neighbor_pos(&entry.pos, &neigh) {
                        let light_level = world.get_light_level(&neigh_pos, self.channel.get());
                        let block_lid = world.get_block(&neigh_pos);
                        let block = blocks_loader.get_block(block_lid);
                        if (!block.mesh.is_cube()) && (entry.light_level > (light_level + 1)) {
                            world.set_light_level(&neigh_pos, self.channel.get(), (entry.light_level - 1));
                            let neigh_entry: LightEntry = LightEntry::new(&neigh_pos, (entry.light_level - 1));
                            add_queue.push_back(neigh_entry);
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
}
