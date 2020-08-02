pub mod instruction;
pub mod register;
use crate::device::memory::Memory;
use crate::device::Device;
use register::Register;

#[cfg(test)]
use std::collections::HashMap;

pub struct CPU {
    memory: Box<dyn Device>,
    registers: Memory,
    stack_frame_size: u16,
}

impl CPU {
    pub fn new(memory: Box<dyn Device>) -> CPU {
        let mut cpu = CPU {
            memory,
            registers: Memory::new(register::SIZE),
            stack_frame_size: 0,
        };
        cpu.set_register(register::SP, cpu.memory.len() as u16 - 2);
        cpu.set_register(register::FP, cpu.memory.len() as u16 - 2);
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

    fn execute(&mut self, instruction: u8) -> bool {
        match instruction {
            instruction::MOVE_LIT_MEM => {
                let value = self.fetch16();
                let mem = self.fetch16();
                self.memory.set_u16(mem as usize, value)
            }
            instruction::MOVE_LIT_REG => {
                let value = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(reg, value)
            }
            instruction::MOVE_REG_REG => {
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                self.set_register(reg_to, self.get_register(reg_from))
            }
            instruction::MOVE_REG_PTR_REG => {
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                let ptr = self.get_register(reg_from);
                let val = self.memory.get_u16(ptr as usize);
                self.set_register(reg_to, val)
            }
            instruction::MOVE_LIT_OFF_REG => {
                let address = self.fetch16();
                let reg_from = self.fetch_register_index();
                let reg_to = self.fetch_register_index();
                let offset = self.get_register(reg_from);
                let val = self.memory.get_u16((offset + address) as usize);
                self.set_register(reg_to, val)
            }
            instruction::MOVE_REG_MEM => {
                let reg = self.fetch_register_index();
                let mem = self.fetch16();
                self.memory.set_u16(mem as usize, self.get_register(reg))
            }
            instruction::MOVE_MEM_REG => {
                let mem = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(reg, self.memory.get_u16(mem as usize))
            }

            instruction::REG_ADD => {
                let r1 = self.fetch_register_index();
                let r2 = self.fetch_register_index();
                self.set_register(register::ACC, self.get_register(r1) + self.get_register(r2))
            }
            instruction::ADD_LIT_REG => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, self.get_register(reg) + val)
            }
            instruction::SUB_LIT_REG => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, val - self.get_register(reg))
            }
            instruction::SUB_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.set_register(register::ACC, self.get_register(reg) - val)
            }
            instruction::SUB_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.set_register(
                    register::ACC,
                    self.get_register(reg_1) - self.get_register(reg_2),
                )
            }
            instruction::MUL_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.set_register(
                    register::ACC,
                    self.get_register(reg_1) * self.get_register(reg_2),
                )
            }
            instruction::MUL_LIT_REG => {
                let val = self.fetch16();
                let reg = self.fetch_register_index();
                self.set_register(register::ACC, val * self.get_register(reg))
            }
            instruction::INC_REG => {
                let reg = self.fetch_register_index();
                self.registers.set_u16(reg, self.get_register(reg) + 1);
            }
            instruction::DEC_REG => {
                let reg = self.fetch_register_index();
                self.registers.set_u16(reg, self.get_register(reg) - 1);
            }

            // Binary operations
            instruction::LST_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers
                    .set_u16(reg_1, self.get_register(reg_1) << self.get_register(reg_2))
            }
            instruction::LST_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers.set_u16(reg, self.get_register(reg) << val)
            }
            instruction::RST_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers
                    .set_u16(reg_1, self.get_register(reg_1) >> self.get_register(reg_2))
            }
            instruction::RST_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers.set_u16(reg, self.get_register(reg) >> val)
            }
            instruction::AND_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) & self.get_register(reg_2),
                )
            }
            instruction::AND_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) & val)
            }
            instruction::OR_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) | self.get_register(reg_2),
                )
            }
            instruction::OR_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) | val)
            }
            instruction::XOR_REG_REG => {
                let reg_1 = self.fetch_register_index();
                let reg_2 = self.fetch_register_index();
                self.registers.set_u16(
                    register::ACC,
                    self.get_register(reg_1) ^ self.get_register(reg_2),
                )
            }
            instruction::XOR_REG_LIT => {
                let reg = self.fetch_register_index();
                let val = self.fetch16();
                self.registers
                    .set_u16(register::ACC, self.get_register(reg) ^ val)
            }
            instruction::NOT => {
                let reg = self.fetch_register_index();
                self.registers
                    .set_u16(register::ACC, !self.get_register(reg))
            }

            // Conditional jumps
            instruction::JNE_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) != lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JNE_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) != self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JEQ_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) == lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JEQ_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) == self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JGT_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) > lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JGT_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) > self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JLT_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) < lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JLT_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) < self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JGE_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) >= lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JGE_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) >= self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JLE_LIT => {
                let lit = self.fetch16();
                let address = self.fetch16();
                if self.get_register(register::ACC) <= lit {
                    self.set_register(register::IP, address)
                }
            }
            instruction::JLE_REG => {
                let reg = self.fetch_register_index();
                let address = self.fetch16();
                if self.get_register(register::ACC) <= self.get_register(reg) {
                    self.set_register(register::IP, address)
                }
            }

            instruction::PUSH_LIT => {
                let lit = self.fetch16();
                self.push_to_stack(lit);
            }
            instruction::PUSH_REG => {
                let reg = self.fetch_register_index();
                self.push_to_stack(self.get_register(reg));
            }
            instruction::POP => {
                let reg = self.fetch_register_index();
                let value = self.pop_from_stack();
                self.set_register(reg, value);
            }
            instruction::CAL_LIT => {
                let address = self.fetch16();
                self.push_state();
                self.set_register(register::IP, address);
            }
            instruction::CAL_REG => {
                let reg = self.fetch_register_index();
                let address = self.get_register(reg);
                self.push_state();
                self.set_register(register::IP, address);
            }
            instruction::RET => {
                self.pop_state();
            }
            instruction::HLT => return true,
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
    use super::instruction;
    use super::register;
    use super::CPU;
    use crate::device::memory::Memory;
    use crate::device::Device;

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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_LIT_REG);
        mem.set_u16(5, 0xABCD);
        mem.set_u8(7, register::R2 as u8);
        mem.set_u8(8, instruction::REG_ADD);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_REG_REG);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::MOVE_REG_MEM);
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
        mem.set_u8(0, instruction::MOVE_LIT_MEM);
        mem.set_u16(1, 0x1234);
        mem.set_u16(3, 0x6);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();

        assert_eq!(cpu.memory.get_u16(0x6), 0x1234);
    }

    #[test]
    fn move_reg_ptr_reg() {
        let mut mem = Memory::new(8);
        mem.set_u8(0, instruction::MOVE_REG_PTR_REG);
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
        mem.set_u8(0, instruction::MOVE_LIT_OFF_REG);
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
        mem.set_u8(0, instruction::ADD_LIT_REG);
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
        mem.set_u8(0, instruction::SUB_LIT_REG);
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
        mem.set_u8(0, instruction::SUB_REG_LIT);
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
        mem.set_u8(0, instruction::SUB_REG_REG);
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
        mem.set_u8(0, instruction::MUL_REG_REG);
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
        mem.set_u8(0, instruction::MUL_LIT_REG);
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
        mem.set_u8(0, instruction::LST_REG_LIT);
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
        mem.set_u8(0, instruction::LST_REG_REG);
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
        mem.set_u8(0, instruction::RST_REG_LIT);
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
        mem.set_u8(0, instruction::RST_REG_REG);
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
        mem.set_u8(0, instruction::AND_REG_LIT);
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
        mem.set_u8(0, instruction::AND_REG_REG);
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
        mem.set_u8(0, instruction::OR_REG_LIT);
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
        mem.set_u8(0, instruction::OR_REG_REG);
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
        mem.set_u8(0, instruction::XOR_REG_LIT);
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
        mem.set_u8(0, instruction::XOR_REG_REG);
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
        mem.set_u8(0, instruction::NOT);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::ACC), 0xfffd);
    }

    #[test]
    fn inc_reg() {
        let mut mem = Memory::new(2);
        mem.set_u8(0, instruction::INC_REG);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x3);
    }

    #[test]
    fn dec_reg() {
        let mut mem = Memory::new(2);
        mem.set_u8(0, instruction::DEC_REG);
        mem.set_u8(1, register::R1 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.set_register(register::R1, 0x2);
        cpu.step();

        assert_eq!(cpu.get_register(register::R1), 0x1);
    }

    #[test]
    fn move_mem_reg() {
        let mut mem = Memory::new(4);
        mem.set_u8(0, instruction::MOVE_MEM_REG);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, register::ACC as u8);
        mem.set_u8(4, instruction::JNE_LIT);
        mem.set_u16(5, 0x1234);
        mem.set_u16(7, 0x0);
        mem.set_u8(9, instruction::JNE_LIT);
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
        mem.set_u8(0, instruction::PUSH_LIT);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 0xABCD);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::PUSH_REG);
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
        mem.set_u8(0, instruction::PUSH_LIT);
        mem.set_u16(1, 0x1234);
        mem.set_u8(3, instruction::POP);
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
        mem.set_u8(0, instruction::CAL_LIT);
        mem.set_u16(1, 10);
        mem.set_u8(10, instruction::MOVE_LIT_REG);
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
        mem.set_u8(0, instruction::MOVE_LIT_REG);
        mem.set_u16(1, 10);
        mem.set_u8(3, register::R1 as u8);
        mem.set_u8(4, instruction::CAL_REG);
        mem.set_u8(5, register::R1 as u8);
        mem.set_u8(10, instruction::MOVE_LIT_REG);
        mem.set_u16(11, 0x3333);
        mem.set_u8(13, register::R2 as u8);

        let mut cpu = CPU::new(Box::new(mem));
        cpu.step();
        cpu.step();
        cpu.step();
        let r2 = cpu.get_register(register::R2);
        assert_eq!(r2, 0x3333);
    }
}