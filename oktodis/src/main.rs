extern crate clap;
extern crate okto;

use clap::{App, Arg};

use okto::cpu;
use okto::memory;
use okto::read_rom_file;
use okto::OktoResult;

/// Display the disassembly of the given ROM file. An offset can be provided so
/// that, for example, a ROM with an initial data section can be correctly
/// disassembled.
fn print_disassembly(rom_data: &Vec<u8>, offset: usize) -> OktoResult<()> {
    let mut memory = memory::Memory::new();

    memory.load(rom_data, cpu::DEFAULT_PC_ADDRESS, rom_data.len())?;

    let num_instructions = (rom_data.len() - offset) / 2;
    for addr in 0..num_instructions {
        let next_address =
            cpu::DEFAULT_PC_ADDRESS + offset as cpu::Address + (2 * addr) as cpu::Address;
        if let Some(instruction) = memory.read_instruction(next_address) {
            if let Some(operation) = cpu::Operation::from_instruction(&instruction) {
                println!("{:03X} {:04X} {:?}", next_address, instruction, operation);
            } else {
                println!("{:03X} {:04X} UNKNOWN", next_address, instruction);
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let matches = App::new("oktodis")
        .version("1.0")
        .author("Eric Scrivner <eric.t.scrivner@gmail.com>")
        .about("Disassembles and displays Chip8 ROM assembly code")
        .arg(
            Arg::with_name("ROMFILE")
                .help("Path to the Chip8 ROM file.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("offset")
                .short("o")
                .long("offset")
                .value_name("NUMBYTES")
                .help("number of bytes in ROM at which to start disassembly")
                .takes_value(true),
        )
        .get_matches();

    let rom_path = matches.value_of("ROMFILE").unwrap();
    let offset = matches
        .value_of("offset")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap();
    let rom_data = read_rom_file(rom_path)?;

    if rom_data.len() > memory::MAX_ROM_SIZE_BYTES {
        println!("File is too large to be a valid Chip8 ROM.");
        return Err(io::Error::new(io::ErrorKind::InvalidData, "ROM too large"));
    }

    print_disassembly(&rom_data, offset).unwrap();

    Ok(())
}
