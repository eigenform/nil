
extern crate elf;

use nil::Jit;
use nil::guest::*;
use nil::block::*;

use std::collections::HashMap;

pub fn main() {
    let arg: Vec<String> = std::env::args().collect();
    let elf = match elf::File::open_path(&arg[1]) {
        Ok(f) => f,
        Err(e) => panic!("{:?}", e),
    };

    // Setup the initial state
    let mut jit = Jit::new();
    jit.state.reg[11] = 0xdead_0011;
    jit.state.reg[13] = 0x0000_8000;
    jit.state.reg[14] = 0xdead_0014;
    jit.state.pc      = ProgramCounter(elf.ehdr.entry as u32);

    // Load an ELF into memory
    for s in elf.sections.iter() {
        if s.shdr.size == 0 { continue }
        match s.shdr.name.as_str() {
            ".symtab" | ".strtab" | ".shstrtab" => continue,
            _ => {
                jit.mmu.write_buf(s.shdr.addr as u32, &s.data);
                println!("LOAD section {}\t ({:08x} bytes) @ {:08x}",
                    s.shdr.name, s.shdr.size as u32, s.shdr.addr as u32);
            },
        }
    }

    // Just run until we terminate
    jit.run();

}
