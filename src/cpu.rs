#[cfg(test)]
use std::collections::HashMap;

use register::Register;

use crate::device::memory::Memory;
use crate::device::Device;

pub mod instruction;
pub mod register;

pub struct CPU {
    memory: Box<dyn Device>,
    registers: Memory,
    stack_frame_size: u16,
    is_in_interrupt_handler: bool,
}

const INTERRUPT_VECTOR_ADDRESS: usize = 0x1000;

impl CPU {
    pub fn new(memory: Box<dyn Device>) -> CPU {
        let mut cpu = CPU {
            memory,
            registers: Memory::new(register::SIZE),
            stack_frame_size: 0,
            is_in_interrupt_handler: false,
        };
        cpu.set_register(register::SP, cpu.memory.len() as u16 - 2);
        cpu.set_register(register::FP, cpu.memory.len() as u16 - 2);
        cpu.set_register(register::IM, 0xff);
        cpu
    }

    pub fn run(&mut self) {
        while !self.step() {}
    }

    #[cfg(test)]
    fn debug_registers(&self) -> HashMap<Register, u16> {
        let mut res = HashMap::new();
        for &reg in register::LIST.iter() {
            res.insert(reg, self.registers.get_u16(reg as usize));
        }
        res
    }

    fn set_register(&mut self, reg: Register, value: u16) {
        if reg == register::MB {
            self.memory.set_mb(value)
        }
        self.registers.set_u16(reg, value);
    }

    fn get_register(&self, reg: Register) -> u16 {
        self.registers.get_u16(reg)
    }

    fn fetch8(&mut self) -> u8 {
        let ip = self.get_register(register::IP);
        let res = self.memory.get_u8(ip as usize);
        self.set_register(register::IP, ip + 1);
        res
    }

    fn fetch16(&mut self) -> u16 {
        let ip = self.get_register(register::IP);
        let res = self.memory.get_u16(ip as usize);
        self.set_register(register::IP, ip + 2);
        res
    }

    fn push_to_stack(&mut self, value: u16) {
        let sp = self.get_register(register::SP);
        self.memory.set_u16(sp as usize, value);
        self.set_register(register::SP, sp - 2);
        self.stack_frame_size += 2;
    }

    fn pop_from_stack(&mut self) -> u16 {
        let new_sp_address = self.get_register(register::SP) + 2;
        self.set_register(register::SP, new_sp_address);
        self.stack_frame_size -= 2;
        self.memory.get_u16(new_sp_address as usize)
    }

    fn fetch_register_index(&mut self) -> Register {
        self.fetch8() as usize
    }

    fn push_state(&mut self) {
        for &reg in register::GENERAL_PURPOSE_LIST.iter() {
            self.push_to_stack(self.get_register(reg));
        }
        self.push_to_stack(self.get_register(register::IP));
        self.push_to_stack(self.stack_frame_size + 2);
        self.set_register(register::FP, self.get_register(register::SP));
        self.stack_frame_size = 0;
    }

    fn pop_state(&mut self) {
        let frame_pointer_address = self.get_register(register::FP);
        self.set_register(register::SP, frame_pointer_address);
        self.stack_frame_size = 2; //Hack to make popping work with stack_frame_size == 0

        let stack_frame_size = self.pop_from_stack();
        self.stack_frame_size = stack_frame_size;

        let ip = self.pop_from_stack();
        self.set_register(register::IP, ip);

        for &reg in register::GENERAL_PURPOSE_LIST.iter().rev() {
            let value = self.pop_from_stack();
            self.set_register(reg, value);
        }

        // let n_args = self.pop_from_stack();
        // for _ in 0..n_args {
        //     self.pop_from_stack();
        // }

        self.set_register(register::FP, frame_pointer_address + stack_frame_size);
    }

    fn handle_interrupt(&mut self, value: u16) {
        if (1 << value) & self.get_register(register::IM) == 0 {
            return;
        }
        let address_pointer = INTERRUPT_VECTOR_ADDRESS + (value as usize) * 2;
        let address = self.memory.get_u16(address_pointer);

        if !self.is_in_interrupt_handler {
            self.push_state();
        }

        self.is_in_interrupt_handler = true;
        self.set_register(register::IP, address)
    }

    fn execute(&mut self, instruction: u8) -> bool {
        match instruction {
            x if x == instruction::INT.opcode => {
                let value = self.fetch16();
                self.handle_interrupt(value);
            }
            x if x == instruction::RET_INT.opcode => {
                self.is_in_interrupt_handler = false;
                self.pop_from_stack();
            }
            x if x == instruction::MOVE_LIT_MEM.opcode => {
                let value = self.fetch16();
                let mem = self.fetch16();
                self.memory.set_u16(mem as usize, value)
            }
            x if x == instruction::MOVE_LIT_REG.opcode => {
                let value = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(reg, value)
            }
            x if x == instruction::MOVE_REG_REG.opcode => {
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                self.set_register(reg_to, self.get_register(reg_from))
            }
            x if x == instruction::MOVE_REG_PTR_REG.opcode => {
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                let ptr = self.get_register(reg_from);
                let val = self.memory.get_u16(ptr as usize);
                self.set_register(reg_to, val)
            }
            x if x == instruction::MOVE_LIT_OFF_REG.opcode => {
                let address = self.fetch16();
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                let offset = self.get_register(reg_from);
                let val = self.memory.get_u16((offset + address) as usize);
                self.set_register(reg_to, val)
            }
            x if x == instruction::MOVE_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let mem = self.fetch16();
                self.memory.set_u16(mem as usize, self.get_register(reg))
            }
            x if x == instruction::MOVE_MEM_REG.opcode => {
                let mem = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(reg, self.memory.get_u16(mem as usize))
            }

            x if x == instruction::ADD_REG_REG.opcode => {
                let r1 = self.fetch_register_index();
                let r2 = self.fetch_register_index();
                self.set_register(register::ACC, self.get_register(r1) + self.get_register(r2))
            }
            x if x == instruction::ADD_LIT_REG.opcode => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, self.get_register(reg) + val)
            }
            x if x == instruction::SUB_LIT_REG.opcode => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, val - self.get_register(reg))
            }
            x if x == instruction::SUB_REG_LIT.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.set_register(register::ACC, self.get_register(reg) - val)
            }
            x if x == instruction::SUB_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.set_register(
                    register::ACC,
                    self.get_register(reg_1) - self.get_register(reg_2),
                )
            }
            x if x == instruction::MUL_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.set_register(
                    register::ACC,
                    self.get_register(reg_1) * self.get_register(reg_2),
                )
            }
            x if x == instruction::MUL_LIT_REG.opcode => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, val * self.get_register(reg))
            }
            x if x == instruction::INC_REG.opcode => {
                let reg = self.fetch_register_index();
                self.registers.set_u16(reg, self.get_register(reg) + 1);
            }
            x if x == instruction::DEC_REG.opcode => {
                let reg = self.fetch_register_index();
                self.registers.set_u16(reg, self.get_register(reg) - 1);
            }

            // Binary operations
            x if x == instruction::LSF_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers
                    .set_u16(reg_1, self.get_register(reg_1) << self.get_register(reg_2))
            }
            x if x == instruction::LSF_REG_LIT8.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers.set_u16(reg, self.get_register(reg) << val)
            }
            x if x == instruction::RSF_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers
                    .set_u16(reg_1, self.get_register(reg_1) >> self.get_register(reg_2))
            }
            x if x == instruction::RSF_REG_LIT8.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers.set_u16(reg, self.get_register(reg) >> val)
            }
            x if x == instruction::AND_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) & self.get_register(reg_2),
                )
            }
            x if x == instruction::AND_REG_LIT.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) & val)
            }
            x if x == instruction::OR_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) | self.get_register(reg_2),
                )
            }
            x if x == instruction::OR_REG_LIT.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) | val)
            }
            x if x == instruction::XOR_REG_REG.opcode => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) ^ self.get_register(reg_2),
                )
            }
            x if x == instruction::XOR_REG_LIT.opcode => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) ^ val)
            }
            x if x == instruction::NOT_REG.opcode => {
                let reg = self.fetch_register_index();
                self.registers
                    .set_u16(register::ACC, !self.get_register(reg))
            }

            // Conditional jumps
            x if x == instruction::JNE_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) != lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JNE_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) != self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JEQ_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) == lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JEQ_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) == self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JGT_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) > lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JGT_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) > self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JLT_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) < lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JLT_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) < self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JGE_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) >= lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JGE_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) >= self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JLE_LIT_MEM.opcode => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) <= lit {
                    self.set_register(register::IP, address)
                }
            }
            x if x == instruction::JLE_REG_MEM.opcode => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) <= self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }

            x if x == instruction::PSH_LIT.opcode => {
                let lit = self.fetch16();
                self.push_to_stack(lit);
            }
            x if x == instruction::PSH_REG.opcode => {
                let reg = self.fetch_register_index();
                self.push_to_stack(self.get_register(reg));
            }
            x if x == instruction::POP_REG.opcode => {
                let reg = self.fetch_register_index();
                let value = self.pop_from_stack();
                self.set_register(reg, value);
            }
            x if x == instruction::CAL_LIT.opcode => {
                let address = self.fetch16();
                self.push_state();
                self.set_register(register::IP, address);
            }
            x if x == instruction::CAL_REG.opcode => {
                let reg = self.fetch_register_index();
                let address = self.get_register(reg);
                self.push_state();
                self.set_register(register::IP, address);
            }
            x if x == instruction::RET.opcode => {
                self.pop_state();
            }
            x if x == instruction::HLT.opcode => return true,
            _ => panic!("Unrecognized instruction: {}", instruction),
        }
        false
    }

    fn step(&mut self) -> bool {
        let instruction = self.fetch8();
        self.execute(instruction)
    }
}

#[cfg(test)]
mod tests {
    use crate::device::banked_memory::BankedMemory;
    use crate::device::memory::Memory;
    use crate::device::memory_mapper::MemoryMapper;
    use crate::device::Device;

    use super::instruction;
    use super::register;
    use super::CPU;

    fn view_memory_at(mem: Memory, address: usize) {
        print!("{:X}:", address);
        for byte in address..address + 8 {
            print!(" {:X}", mem.get_u16(byte))
        }
        println!();
    }

    #[test]
    fn push_to_stack() {
        let mem = Memory::new(12);
        let mut cpu = CPU::new(Box::new(mem));
        assert_eq!(cpu.stack_frame_size, 0);
        assert_eq!(cpu.get_register(register::SP), 10);
        assert_eq!(cpu.get_register(register::FP), 10);
        cpu.push_to_stack(111);
        assert_eq!(cpu.stack_frame_size, 2);
        assert_eq!(cpu.get_register(register::SP), 8);
        assert_eq!(cpu.get_register(register::FP), 10);
        assert_eq!(cpu.memory.get_u16(10), 111);
        cpu.push_to_stack(222);
        assert_eq!(cpu.stack_frame_size, 4);
        assert_eq!(cpu.get_register(register::SP), 6);
        assert_eq!(cpu.get_register(register::FP), 10);
        assert_eq!(cpu.memory.get_u16(10), 111);
        assert_eq!(cpu.memory.get_u16(8), 222);
    }

    #[test]
    fn pop_from_stack() {
        let mem = Memory::new(12);
        let mut cpu = CPU::new(Box::new(mem));
        cpu.push_to_stack(111);
        cpu.push_to_stack(222);
        assert_eq!(cpu.stack_frame_size, 4);
        assert_eq!(cpu.get_register(register::SP), 6);
        assert_eq!(cpu.memory.get_u16(10), 111);
        assert_eq!(cpu.memory.get_u16(8), 222);

        let last = cpu.pop_from_stack();
        assert_eq!(cpu.stack_frame_size, 2);
        assert_eq!(cpu.get_register(register::SP), 8);
        assert_eq!(last, 222);

        let first = cpu.pop_from_stack();
        assert_eq!(cpu.stack_frame_size, 0);
        assert_eq!(cpu.get_register(register::SP), 10);
        assert_eq!(first, 111);
    }

    #[test]
    fn push_state() {
        let mem = Memory::new(64);
        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 20);
        cpu.set_register(register::R4, 30);

        assert_eq!(cpu.stack_frame_size, 0);
        assert_eq!(cpu.get_register(register::SP), 62);
        assert_eq!(cpu.get_register(register::FP), 62);

        cpu.push_state();
        assert_eq!(cpu.stack_frame_size, 0);
        assert_eq!(cpu.get_register(register::SP), 42);
        assert_eq!(cpu.get_register(register::FP), 42);
        assert_eq!(cpu.memory.get_u16(62), 20); //R1
        assert_eq!(cpu.memory.get_u16(56), 30); //R4
        assert_eq!(cpu.memory.get_u16(44), 20); //stack frame size
        cpu.set_register(register::R4, 40);
        cpu.set_register(register::R3, 50);

        cpu.push_state();
        assert_eq!(cpu.stack_frame_size, 0);
        assert_eq!(cpu.get_register(register::SP), 22);
        assert_eq!(cpu.get_register(register::FP), 22);
        assert_eq!(cpu.memory.get_u16(62), 20); //R1
        assert_eq!(cpu.memory.get_u16(56), 30); //R4
        assert_eq!(cpu.memory.get_u16(44), 20); //stack frame size
        assert_eq!(cpu.memory.get_u16(42), 20); //R1
        assert_eq!(cpu.memory.get_u16(38), 50); //R3
        assert_eq!(cpu.memory.get_u16(36), 40); //R4
        assert_eq!(cpu.memory.get_u16(24), 20); //stack frame size
    }

    #[test]
    fn pop_state() {
        let mem = Memory::new(64);
        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 20);
        cpu.set_register(register::R4, 30);

        cpu.push_state();
        cpu.set_register(register::R4, 40);
        cpu.set_register(register::R3, 50);

        cpu.push_state();
        assert_eq!(cpu.get_register(register::SP), 22);
        assert_eq!(cpu.get_register(register::FP), 22);
        cpu.set_register(register::R4, 60);
        cpu.set_register(register::R2, 70);
        assert_eq!(cpu.get_register(register::R1), 20);
        assert_eq!(cpu.get_register(register::R2), 70);
        assert_eq!(cpu.get_register(register::R3), 50);
        assert_eq!(cpu.get_register(register::R4), 60);

        cpu.pop_state();
        assert_eq!(cpu.get_register(register::SP), 42);
        assert_eq!(cpu.get_register(register::FP), 42);
        assert_eq!(cpu.get_register(register::R1), 20);
        assert_eq!(cpu.get_register(register::R2), 0);
        assert_eq!(cpu.get_register(register::R3), 50);
        assert_eq!(cpu.get_register(register::R4), 40);

        cpu.pop_state();
        assert_eq!(cpu.get_register(register::SP), 62);
        assert_eq!(cpu.get_register(register::FP), 62);
        assert_eq!(cpu.get_register(register::R1), 20);
        assert_eq!(cpu.get_register(register::R2), 0);
        assert_eq!(cpu.get_register(register::R3), 0);
        assert_eq!(cpu.get_register(register::R4), 30);
    }

    #[test]
    fn reg_add() {
        let mut mem = Memory::new(11);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(5, 0xABCD);
        mem.set_u8(7, register::R2 as u8);
        mem.set_u8(8, instruction::ADD_REG_REG.opcode);
        mem.set_u8(9, register::R1 as u8);
        mem.set_u8(10, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));

        cpu.step();
        assert_eq!(cpu.debug_registers()[&register::R1], 0x1234);
        cpu.step();
        assert_eq!(cpu.debug_registers()[&register::R2], 0xABCD);
        cpu.step();
        assert_eq!(cpu.debug_registers()[&register::ACC], 0xBE01);
    }

    #[test]
    fn move_lit_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x1234);
        assert_eq!(cpu.registers.get_u8(register::R1), 0x12);
        assert_eq!(cpu.registers.get_u8(register::R1 + 1), 0x34);
    }

    #[test]
    fn move_reg_reg() {
        let mut mem = Memory::new(7);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_REG_REG.opcode);
        mem.set_u8(5, register::R1 as u8);
        mem.set_u8(6, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();

        assert_eq!(cpu.registers.get_u16(register::R2), 0x1234);
        assert_eq!(cpu.registers.get_u8(register::R2), 0x12);
        assert_eq!(cpu.registers.get_u8(register::R2 + 1), 0x34);
    }

    #[test]
    fn move_reg_mem() {
        let mut mem = Memory::new(8);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_REG_MEM.opcode);
        mem.set_u8(5, register::R1 as u8);
        mem.set_u16(6, 0x1);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();

        assert_eq!(cpu.memory.get_u16(0x1), 0x1234);
        assert_eq!(cpu.memory.get_u8(0x1), 0x12);
        assert_eq!(cpu.memory.get_u8(0x1 + 1), 0x34);
    }

    #[test]
    fn move_lit_mem() {
        let mut mem = Memory::new(8);
        mem.set_u8(0, instruction::MOVE_LIT_MEM.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u16(3, 0x6);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();

        assert_eq!(cpu.memory.get_u16(0x6), 0x1234);
    }

    #[test]
    fn move_reg_ptr_reg() {
        let mut mem = Memory::new(8);
        mem.set_u8(0, instruction::MOVE_REG_PTR_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);
        mem.set_u16(0x6, 0x5555);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x6);
        cpu.step();

        assert_eq!(cpu.get_register(register::R2), 0x5555);
    }

    #[test]
    fn move_reg_off_reg() {
        let mut mem = Memory::new(8);
        mem.set_u8(0, instruction::MOVE_LIT_OFF_REG.opcode);
        mem.set_u16(1, 1);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, register::R2 as u8);
        mem.set_u16(0x6, 0x5555);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x5);
        cpu.step();

        assert_eq!(cpu.get_register(register::R2), 0x5555);
    }

    #[test]
    fn add_lit_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::ADD_LIT_REG.opcode);
        mem.set_u16(1, 5);
        mem.set_u8(3, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x5);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xa);
    }

    #[test]
    fn sub_lit_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::SUB_LIT_REG.opcode);
        mem.set_u16(1, 5);
        mem.set_u8(3, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x3);
    }

    #[test]
    fn sub_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::SUB_REG_LIT.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 5);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0xe);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x9);
    }

    #[test]
    fn sub_reg_reg() {
        let mut mem = Memory::new(3);
        mem.set_u8(0, instruction::SUB_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0xe);
        cpu.set_register(register::R2, 0x6);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x8);
    }

    #[test]
    fn mul_reg_reg() {
        let mut mem = Memory::new(3);
        mem.set_u8(0, instruction::MUL_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.set_register(register::R2, 0x6);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xc);
    }

    #[test]
    fn mul_lit_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::MUL_LIT_REG.opcode);
        mem.set_u16(1, 0x3);
        mem.set_u8(3, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x6);
    }

    #[test]
    fn lst_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::LSF_REG_LIT8.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 0x3);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x10);
    }

    #[test]
    fn lst_reg_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::LSF_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.set_register(register::R2, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x8);
    }

    #[test]
    fn rst_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::RSF_REG_LIT8.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 0x1);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x1);
    }

    #[test]
    fn rst_reg_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::RSF_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x8);
        cpu.set_register(register::R2, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x2);
    }

    #[test]
    fn and_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::AND_REG_LIT.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 0x1);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x3);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x1);
    }

    #[test]
    fn and_reg_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::AND_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0xa);
        cpu.set_register(register::R2, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x2);
    }

    #[test]
    fn or_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::OR_REG_LIT.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 0x8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x3);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xb);
    }

    #[test]
    fn or_reg_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::OR_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0xa);
        cpu.set_register(register::R2, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xa);
    }

    #[test]
    fn xor_reg_lit() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::XOR_REG_LIT.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u16(2, 0x1);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x3);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x2);
    }

    #[test]
    fn xor_reg_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::XOR_REG_REG.opcode);
        mem.set_u8(1, register::R1 as u8);
        mem.set_u8(2, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0xa);
        cpu.set_register(register::R2, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0x8);
    }

    #[test]
    fn not() {
        let mut mem = Memory::new(2);
        mem.set_u8(0, instruction::NOT_REG.opcode);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xfffd);
    }

    #[test]
    fn inc_reg() {
        let mut mem = Memory::new(2);
        mem.set_u8(0, instruction::INC_REG.opcode);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x3);
    }

    #[test]
    fn dec_reg() {
        let mut mem = Memory::new(2);
        mem.set_u8(0, instruction::DEC_REG.opcode);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x1);
    }

    #[test]
    fn move_mem_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::MOVE_MEM_REG.opcode);
        mem.set_u16(1, 0x1);
        mem.set_u8(3, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x1);
        assert_eq!(cpu.registers.get_u8(register::R1), 0x00);
        assert_eq!(cpu.registers.get_u8(register::R1 + 1), 0x01);
    }

    #[test]
    fn jmp_not_eq() {
        let mut mem = Memory::new(14);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::ACC as u8);
        mem.set_u8(4, instruction::JNE_LIT_MEM.opcode);
        mem.set_u16(5, 0x1234);
        mem.set_u16(7, 0x0);
        mem.set_u8(9, instruction::JNE_LIT_MEM.opcode);
        mem.set_u16(10, 0x12AB);
        mem.set_u16(12, 0x2);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        assert_eq!(cpu.get_register(register::ACC), 0x1234);
        cpu.step();
        assert_eq!(cpu.get_register(register::IP), 0x9);
        cpu.step();
        assert_eq!(cpu.get_register(register::IP), 0x2);
    }

    #[test]
    fn push_lit() {
        let mut mem = Memory::new(6);
        mem.set_u8(0, instruction::PSH_LIT.opcode);
        mem.set_u16(1, 0x1234);

        let mut cpu = CPU::new(Box::new(mem));
        let mut sp = cpu.get_register(register::SP);
        assert_eq!(sp, 4);
        cpu.step();
        sp = cpu.get_register(register::SP);
        assert_eq!(sp, 2);
        assert_eq!(cpu.memory.get_u16(sp as usize + 2), 0x1234);
    }

    #[test]
    fn push_reg() {
        let mut mem = Memory::new(10);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 0xABCD);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::PSH_REG.opcode);
        mem.set_u8(5, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();
        let sp = cpu.get_register(register::SP);
        assert_eq!(sp, 6);
        assert_eq!(cpu.memory.get_u16(sp as usize + 2), 0xABCD);
    }

    #[test]
    fn pop() {
        let mut mem = Memory::new(10);
        mem.set_u8(0, instruction::PSH_LIT.opcode);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, instruction::POP_REG.opcode);
        mem.set_u8(4, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        let mut sp = cpu.get_register(register::SP);
        assert_eq!(sp, 8);
        cpu.step();
        sp = cpu.get_register(register::SP);
        assert_eq!(sp, 6);
        cpu.step();
        sp = cpu.get_register(register::SP);
        assert_eq!(sp, 8);
        assert_eq!(cpu.get_register(register::R1), 0x1234);
    }

    #[test]
    fn cal_lit() {
        let mut mem = Memory::new(34);
        mem.set_u8(0, instruction::CAL_LIT.opcode);
        mem.set_u16(1, 10);
        mem.set_u8(10, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(11, 0x3333);
        mem.set_u8(13, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();
        let r1 = cpu.get_register(register::R1);
        assert_eq!(r1, 0x3333);
    }

    #[test]
    fn cal_reg() {
        let mut mem = Memory::new(34);
        mem.set_u8(0, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(1, 10);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::CAL_REG.opcode);
        mem.set_u8(5, register::R1 as u8);
        mem.set_u8(10, instruction::MOVE_LIT_REG.opcode);
        mem.set_u16(11, 0x3333);
        mem.set_u8(13, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();
        cpu.step();
        let r2 = cpu.get_register(register::R2);
        assert_eq!(r2, 0x3333);
    }

    #[test]
    fn banked_memory() {
        let mut mm = MemoryMapper::new();
        let mem = Memory::new(0xff00);
        let mem_bank = BankedMemory::new(8, 256);

        mm.map(Box::new(mem_bank), 0x0000, 0x00ff, false);
        mm.map(Box::new(mem), 0x00ff, 0xffff, true);
        let mut cpu = CPU::new(Box::new(mm));

        cpu.memory.set_u8(123, 0x8);
        assert_eq!(cpu.memory.get_u8(123), 0x8);

        cpu.set_register(register::MB, 1);
        assert_eq!(cpu.memory.get_u8(123), 0);
        cpu.memory.set_u8(123, 0x80);
        assert_eq!(cpu.memory.get_u8(123), 0x80);

        cpu.set_register(register::MB, 0);
        assert_eq!(cpu.memory.get_u8(123), 0x8);
    }
}
