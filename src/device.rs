pub mod memory;
pub mod memory_mapper;
pub mod screen;

pub trait Device {
    fn get_u16(&self, address: usize) -> u16;
    fn get_u8(&self, address: usize) -> u8;
    fn set_u16(&mut self, address: usize, value: u16);
    fn set_u8(&mut self, address: usize, value: u8);
    fn len(&self) -> usize;
}
