extern crate elf;
extern crate sha2;
extern crate term;

use std::env;
use std::boxed::Box;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

struct Cut {
    offset: u64,
    debug: String,
}


struct ElfCutter{
}

impl ElfCutter {
    fn new<T>(mut io: T) -> Result<Box<Iterator<Item=Cut>>, elf::ParseError>  where T: Read + Seek {
        let f = elf::File::open_stream(&mut io)?;

        //if let Some(symtab)  = f.get_section(".dynsym") {
        //    for symbol in f.get_symbols(&symtab).unwrap() {
        //        println!("{} {}", symbol.value, symbol.name);
        //    }
        //}

        let section_iterator = f.sections.into_iter();
        let section_iterator = section_iterator.filter_map(|sec| {
            Some(Cut{
                offset: sec.shdr.offset,
                debug: sec.shdr.name,
            })
        });

        Ok(Box::new(section_iterator))
    }
}

fn main() {

    let mut blocks: HashMap<Vec<u8>, bool> = HashMap::new();
    let mut insize      = 0;
    let mut outsize     = 0;

    for filename in env::args().skip(1) {
        println!("");
        println!("----------------------------------------------");
        let path = PathBuf::from(filename.clone());
        let mut file = File::open(path).unwrap();
        insize += file.metadata().unwrap().len();

        let mut cuts : Vec<Cut> = ElfCutter::new(&mut file).unwrap().collect();
        cuts.push(Cut{
            offset: file.metadata().unwrap().len(),
            debug: String::from("EOF"),
        });


        file.seek(SeekFrom::Start(0)).unwrap();

        let mut previous_block = 0;

        for cut in cuts {
            //Only take cut into account if its further into the file.
            if previous_block >= cut.offset {
                continue;
            }
            let blocksize = cut.offset - previous_block;
            let mut data = vec![0; blocksize as usize];
            file.read(&mut data).unwrap();
            let hash = Sha256::digest(&data);

            let mut t = term::stdout().unwrap();
            if let None = blocks.insert(hash.as_slice().to_vec(), true) {
                outsize += blocksize;
            } else {
                t.fg(term::color::GREEN).unwrap();
            }
            writeln!(t, "{}\t{}\t{:x}\t{}", cut.offset, blocksize, hash, cut.debug).unwrap();
            t.reset().unwrap();

            previous_block = cut.offset;
        }
    }
    println!("==============================================");
    let pc = 100.0 * (outsize as f32 / insize as f32);
    println!("files size: {} after dedup: {} ({:.2}%) blocks: {}", insize, outsize, pc, blocks.len());


}
