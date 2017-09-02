extern crate sha2;
extern crate colored;
#[macro_use]
extern crate elf;
extern crate byteorder;


use std::env;
use std::boxed::Box;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use colored::*;

struct Cut {
    offset: u64,
    debug: String,
}

fn symbols<T>(mut io: T) -> Result<Box<Iterator<Item=Cut>>, elf::ParseError>  where T: Read + Seek {
    let f = elf::File::open_stream(&mut io)?;

    let mut symtab_section_offset = 0;

    let section_iterator = f.sections.iter();
    let section_iterator = section_iterator.flat_map(|sec| {
        match &sec.shdr.name as &str {
            ".symtab" => {
                let mut cuts = Vec::new();
                cuts.push(Cut{
                    offset: sec.shdr.offset,
                    debug:  sec.shdr.name.clone(),
                });
                for sym in f.get_symbols(&sec).unwrap() {
                    match sym.symtype {
                        elf::types::STT_SECTION => {
                            symtab_section_offset = sym.value;
                        },
                        elf::types::STT_FUNC | elf::types::STT_OBJECT | elf::types::STT_NOTYPE => {
                            cuts.push(Cut {
                                offset: sym.value - symtab_section_offset,
                                debug:  String::from("  ") + &sym.name,
                            });
                        },
                        _ => {
                            //println!("{}", sym);
                        }
                    }
                };
                cuts.into_iter()
            }
            &_ => {vec![].into_iter()},
        }
    }).collect::<Vec<Cut>>().into_iter();

    Ok(Box::new(section_iterator))
}

fn sections<T>(mut io: T) -> Result<Box<Iterator<Item=Cut>>, elf::ParseError>  where T: Read + Seek {
    let f = elf::File::open_stream(&mut io)?;

    let mut symtab_section_offset = 0;

    let section_iterator = f.sections.iter();
    let section_iterator = section_iterator.flat_map(|sec| {
        match &sec.shdr.name as &str {
            &_ => {
                vec![Cut{
                    offset: sec.shdr.offset,
                    debug:  sec.shdr.name.clone(),
                }].into_iter()
            }
            &_ => {vec![].into_iter()},
        }
    }).collect::<Vec<Cut>>().into_iter();

    Ok(Box::new(section_iterator))
}


fn main() {

    let mut blocks: HashMap<Vec<u8>, bool> = HashMap::new();
    let mut insize   = 0;
    let mut outsize  = 0;

    let mut filei = 0;
    for filename in env::args().skip(1) {

        if filei > 0 {
            println!("");
            println!("|{:<7}|{:<7}|{:64}|{:.20}", "size", "offset", "hash", "symbol");

            println!("-------------------------------------------------------------------------------------------");
        }

        let mut cuts : Vec<Cut> = Vec::new();

        let path = PathBuf::from(filename.clone());
        let mut file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let file_len = file.metadata().unwrap().len();
        insize += file_len;
        cuts.push(Cut{
            offset: file_len,
            debug: String::from("EOF"),
        });


        //read symbols from .d file
        {
            let path = PathBuf::from(filename.clone() + ".d");
            let mut file = OpenOptions::new().read(true).write(true).open(path).unwrap();
            let mut c2 :  Vec<Cut> = symbols(&mut file).unwrap().collect();
            cuts.append(&mut c2);
        }

        //read sections from target file
        let mut c2 :  Vec<Cut> = sections(&mut file).unwrap().collect();
        cuts.append(&mut c2);


        cuts.sort_by(|a, b| a.offset.cmp(&b.offset));

        file.seek(SeekFrom::Start(0)).unwrap();

        let mut previous_block = 0;
        let mut this_debug = String::from("header");

        for cut in cuts {
            //Only take cut into account if its further into the file.
            if previous_block >= cut.offset {
                continue;
            }
            let blocksize = cut.offset - previous_block;
            let mut data = vec![0; blocksize as usize];
            let rlen = file.read(&mut data).unwrap();
            if rlen < blocksize as usize {
                println!("cut beyond EOF: {}. expected {} bytes to {} got {}", cut.debug, blocksize, cut.offset, rlen);
                break;
            }
            let hash = Sha256::digest(&data);

            if let None = blocks.insert(hash.as_slice().to_vec(), true) {
                outsize += blocksize;
                if filei > 0 {
                    print!("{}", "+".red())
                }
            } else {
                if filei > 0 {
                    print!("{}", "=".green())
                }
            }
            if filei > 0 {
                println!("{:<7x}|{:<7x}|{:x}|{:.40}", blocksize, previous_block, hash, this_debug);
            }

            //this marker is actually the start of the next section
            previous_block = cut.offset;
            this_debug     = cut.debug;
        }
        filei +=1;
        println!("==============================================");
        let pc = 100.0 * (outsize as f32 / insize as f32);
        println!("files size: {} after dedup: {} ({:.2}%) blocks: {}", insize, outsize, pc, blocks.len());
    }


}
