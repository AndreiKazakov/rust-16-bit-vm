use crate::device::Device;
use device::memory::Memory;

mod assembler;
mod cpu;
mod device;
mod parser_combinator;

fn main() {
    let mut mm = device::memory_mapper::MemoryMapper::new();

    let mut mem = Memory::new(256 * 256);
    let mut i = 0;

    for (pos, ch) in "Hello world!".chars().enumerate() {
        mem.set_u8(i, cpu::instruction::MOVE_LIT_REG);
        i += 1;
        mem.set_u16(i, ch as u16);
        i += 2;
        mem.set_u8(i, cpu::register::R1 as u8);
        i += 1;

        mem.set_u8(i, cpu::instruction::MOVE_REG_MEM);
        i += 1;
        mem.set_u8(i, cpu::register::R1 as u8);
        i += 1;
        mem.set_u8(i, 0x30);
        i += 1;
        mem.set_u8(i, pos as u8);
        i += 1;
    }

    mem.set_u8(i, cpu::instruction::HLT);

    mm.map(Box::new(mem), 0x0000, 0xffff, false);

    let screen = device::screen::Screen {};
    mm.map(Box::new(screen), 0x3000, 0x30ff, false);

    let mut cpu = cpu::CPU::new(Box::new(mm));
    cpu.run();
}
