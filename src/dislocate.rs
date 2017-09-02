#[macro_use]
extern crate elf;
extern crate byteorder;

use std::io::{Read, Seek, SeekFrom, Write};
use elf::utils;
use elf::types;
use std::env;
use std::path::PathBuf;
use std::fs::OpenOptions;

const R_X86_64_NONE       :u32 = 0;
const R_X86_64_64         :u32 = 1;
const R_X86_64_PC32       :u32 = 2;
const R_X86_64_GOT32      :u32 = 3;
const R_X86_64_PLT32      :u32 = 4;
const R_X86_64_COPY       :u32 = 5;
const R_X86_64_GLOB_DAT   :u32 = 6;
const R_X86_64_JUMP_SLOT  :u32 = 7;
const R_X86_64_RELATIVE   :u32 = 8;
const R_X86_64_GOTPCREL   :u32 = 9;
const R_X86_64_32         :u32 = 10;
const R_X86_64_32S        :u32 = 11;
const R_X86_64_16         :u32 = 12;
const R_X86_64_PC16       :u32 = 13;
const R_X86_64_8          :u32 = 14;
const R_X86_64_PC8        :u32 = 15;
const R_X86_64_DPTMOD64   :u32 = 16;
const R_X86_64_DTPOFF64   :u32 = 17;
const R_X86_64_TPOFF64    :u32 = 18;
const R_X86_64_TLSGD      :u32 = 19;
const R_X86_64_TLSLD      :u32 = 20;
const R_X86_64_DTPOFF32   :u32 = 21;
const R_X86_64_GOTTPOFF   :u32 = 22;
const R_X86_64_TPOFF32    :u32 = 23;
const R_X86_64_PC64       :u32 = 24;
const R_X86_64_GOTOFF64   :u32 = 25;
const R_X86_64_GOTPC32    :u32 = 26;
const R_X86_64_SIZE32     :u32 = 32;
const R_X86_64_SIZE64     :u32 = 33;

pub struct Elf64Rela {
    addr:   u64, //Elf64_Addr
    info:   u64, //Elf64_Xword
    addend: i64, //Elf64_Sxword
}

fn dislocate<T>(mut io: T, f: &elf::File, sec: &elf::Section)  where T: Read + Seek + Write {
    let mut secdata = &sec.data[..];
    while let Ok(offset) = read_u64!(f, secdata) {
        let info         = read_u64!(f, secdata).unwrap();
        let addend       = read_u64!(f, secdata).unwrap();

        let sym   = info >> 32;
        let rtype = (info & 0xffffffff) as u32;

        //println!("relocation: offset: {:x}, symbol: {:x}, type: {:x}, add: {:x}", offset, sym, rtype, addend);
        match rtype {
            0 => {},
            R_X86_64_GOTPCREL | R_X86_64_PC32 | R_X86_64_GOT32 | R_X86_64_PLT32 |
                R_X86_64_TLSGD | R_X86_64_TLSLD | R_X86_64_DTPOFF32 | R_X86_64_GOTTPOFF |
                R_X86_64_32 => {
                io.seek(SeekFrom::Start(offset)).unwrap();
                io.write(&[0;4]).unwrap();
            },
            R_X86_64_64|R_X86_64_JUMP_SLOT|R_X86_64_RELATIVE|R_X86_64_GLOB_DAT|R_X86_64_DTPOFF64 => {
                io.seek(SeekFrom::Start(offset)).unwrap();
                io.write(&[0;8]).unwrap();
            },
            _ => {
                println!("not dislocating reloc type {:?} at {}", rtype, offset);
            },
        }
    }
}

fn main() {
    for filename in env::args().skip(1) {
        let path = PathBuf::from(filename.clone());
        let mut file = OpenOptions::new().read(true).write(true).open(path).unwrap();

        let f = elf::File::open_stream(&mut file).unwrap();
        for sec in &f.sections {

            match sec.shdr.name.as_ref() {
                ".rela.text" | ".rela.rodata" | ".rela.eh_frame" | ".rela.data.rel.ro" => {
                    println!("relocating {}", sec.shdr.name); 
                    dislocate(&file, &f, &sec);
                },
                _ => {},
            }

        }
    }
}
