use crate::device::screen::Screen;
use crate::device::Device;
use device::memory::Memory;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::{env, fs};

mod assembler;
mod cpu;
mod device;
mod parser_combinator;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|command| command.as_str()) {
        Some("compile") => {
            match args.as_slice() {
                [_, _, file, output] => {
                    let bin = assembler::compile(
                        fs::read_to_string(file).map_err(err_to_string)?.as_str(),
                    );
                    let mut file = File::create(output).map_err(err_to_string)?;
                    // Write a slice of bytes to the file
                    file.write_all(&bin).map_err(err_to_string)?;
                }
                _ => return Err("Usage: vm compile <input_file> <output_file>".to_string()),
            };
        }
        Some("run") => {
            if let Some(file) = args.get(2) {
                let mut bin = File::open(file).map_err(err_to_string)?;
                let mut buf = [0u8; 0xfe00];
                bin.read(&mut buf).map_err(err_to_string)?;

                let mem_bank = device::banked_memory::BankedMemory::new(8, 256);
                let screen = Screen {};
                let mut mem = Memory::new(0xff00);

                for i in 0..0xfe00 {
                    mem.set_u8(i, *buf.get(i).ok_or("Mismatched buffer size".to_string())?)
                }

                let mut mm = device::memory_mapper::MemoryMapper::new();
                mm.map(Box::new(mem), 0x0000, 0xfe00, true);
                mm.map(Box::new(screen), 0xfe00, 0xff00, true);
                mm.map(Box::new(mem_bank), 0xff00, 0xffff, false);

                let mut cpu = cpu::CPU::new(Box::new(mm));

                cpu.run()
            } else {
                return Err("Usage: vm run <binary_file>".to_string());
            }
        }
        Some(command) => return Err(format!("{} is not a vm command", command)),
        _ => return Err("Usage: vm <command> [args]".to_string()),
    }

    Ok(())
}

fn err_to_string(err: Error) -> String {
    format!("{:?}", err)
}
