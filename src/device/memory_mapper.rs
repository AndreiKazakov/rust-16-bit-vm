use super::Device;
use std::collections::VecDeque;

struct Region {
    device: Box<dyn Device>,
    start: usize,
    end: usize,
    remap: bool,
}
pub struct MemoryMapper {
    regions: VecDeque<Region>,
}
impl MemoryMapper {
    pub fn new() -> MemoryMapper {
        MemoryMapper {
            regions: VecDeque::new(),
        }
    }

    pub fn map(&mut self, device: Box<dyn Device>, start: usize, end: usize, remap: bool) {
        let region = Region {
            device,
            start,
            end,
            remap,
        };
        self.regions.push_front(region);
    }

    fn find_region(&self, address: usize) -> &Region {
        self.regions
            .iter()
            .find(|region| (region.start..=region.end).contains(&address))
            .unwrap()
    }

    fn find_region_mut(&mut self, address: usize) -> &mut Region {
        self.regions
            .iter_mut()
            .find(|region| (region.start..=region.end).contains(&address))
            .unwrap()
    }
}
impl Device for MemoryMapper {
    fn get_u16(&self, address: usize) -> u16 {
        let region = self.find_region(address);
        region.device.get_u16(if region.remap {
            address - region.start
        } else {
            address
        })
    }

    fn get_u8(&self, address: usize) -> u8 {
        let region = self.find_region(address);
        region.device.get_u8(if region.remap {
            address - region.start
        } else {
            address
        })
    }

    fn set_u16(&mut self, address: usize, value: u16) {
        let region = self.find_region_mut(address);
        region.device.set_u16(
            if region.remap {
                address - region.start
            } else {
                address
            },
            value,
        )
    }

    fn set_u8(&mut self, address: usize, value: u8) {
        let region = self.find_region_mut(address);
        region.device.set_u8(
            if region.remap {
                address - region.start
            } else {
                address
            },
            value,
        )
    }

    fn len(&self) -> usize {
        0xffff
    }

    fn set_mb(&mut self, mb: u16) {
        for region in self.regions.iter_mut() {
            region.device.set_mb(mb)
        }
    }
}
