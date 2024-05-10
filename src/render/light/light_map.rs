use std::cell::{RefCell, RefMut};
use crate::render::types::Vec3ub;

pub struct LightMap {
    pub data: RefCell<[[[u16; 16]; 16]; 16]>, //{4S 4B 4G 4R} - u16 BigEndian
}

pub const R_CHANNEL: u8 = 0;
pub const G_CHANNEL: u8 = 1;
pub const B_CHANNEL: u8 = 2;
pub const S_CHANNEL: u8 = 3;


//TODO: REWRITE XYZ to pos[0] pos[1] pos[2] in LightMap, World and others
impl LightMap {

    pub fn new() -> Self {
        return Self {
            data: RefCell::new([[[0x00_u16; 16]; 16]; 16]),
        };
    }

    pub fn get_r(&self, pos: &Vec3ub) -> u8 {
        return (self.data.borrow()[pos[1] as usize][pos[2] as usize][pos[0] as usize] & 0x0F) as u8;
    }
    pub fn get_g(&self, pos: &Vec3ub) -> u8 {
        return ((self.data.borrow()[pos[1] as usize][pos[2] as usize][pos[0] as usize] >> 4) & 0x0F) as u8;
    }
    pub fn get_b(&self, pos: &Vec3ub) -> u8 {
        return ((self.data.borrow()[pos[1] as usize][pos[2] as usize][pos[0] as usize] >> 8) & 0x0F) as u8;
    }
    pub fn get_s(&self, pos: &Vec3ub) -> u8 {
        return ((self.data.borrow()[pos[1] as usize][pos[2] as usize][pos[0] as usize] >> 12) & 0x0F) as u8;
    }

    pub fn get(&self, pos: &Vec3ub, channel: u8) -> u8 {
        return ((self.data.borrow()[pos[1] as usize][pos[2] as usize][pos[0] as usize] >> (channel << 2)) & 0x0F) as u8;
    }

    pub fn set_r(&self, pos: &Vec3ub, level: u8) {
        let mut data: RefMut<[[[u16; 16]; 16]; 16]> = self.data.borrow_mut();
        data[pos[1] as usize][pos[2] as usize][pos[0] as usize] = (data[pos[1] as usize][pos[2] as usize][pos[0] as usize] & 0xFFF0) | (level as u16);
    }

    pub fn set_g(&self, pos: &Vec3ub, level: u8) {
        let mut data: RefMut<[[[u16; 16]; 16]; 16]> = self.data.borrow_mut();
        data[pos[1] as usize][pos[2] as usize][pos[0] as usize] = (data[pos[1] as usize][pos[2] as usize][pos[0] as usize] & 0xFF0F) | ((level as u16) << 4);
    }

    pub fn set_b(&self, pos: &Vec3ub, level: u8) {
        let mut data: RefMut<[[[u16; 16]; 16]; 16]> = self.data.borrow_mut();
        data[pos[1] as usize][pos[2] as usize][pos[0] as usize] = (data[pos[1] as usize][pos[2] as usize][pos[0] as usize] & 0xF0FF) | ((level as u16) << 8);
    }

    pub fn set_s(&self, pos: &Vec3ub, level: u8) {
        let mut data: RefMut<[[[u16; 16]; 16]; 16]> = self.data.borrow_mut();
        data[pos[1] as usize][pos[2] as usize][pos[0] as usize] = (data[pos[1] as usize][pos[2] as usize][pos[0] as usize] & 0x0FFF) | ((level as u16) << 12);
    }

    pub fn set(&self, pos: &Vec3ub, channel: u8, level: u8) {
        let shift: u8 = channel << 2;
        let mut data: RefMut<[[[u16; 16]; 16]; 16]> = self.data.borrow_mut();
        data[pos[1] as usize][pos[2] as usize][pos[0] as usize] = (data[pos[1] as usize][pos[2] as usize][pos[0] as usize] & (!(0x0F << shift))) | ((level as u16) << shift);
    }

}