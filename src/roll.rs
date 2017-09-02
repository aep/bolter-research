extern crate sha2;
extern crate colored;
extern crate rollsum;
extern crate generic_array;
extern crate digest;
extern crate hex;

use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::collections::HashMap;
use std::io::Read;
use rollsum::Engine;
use sha2::{Sha256, Digest};
use hex::ToHex;
use colored::*;

pub struct Chunker<R,C,H,N> where
R: Read,
C: rollsum::Engine<Digest=u32> + Default,
N: generic_array::ArrayLength<u8>,
H: sha2::Digest<OutputSize=N> + Default
{
    io:      R,
    chunker: C,
    hasher:  H,
    buf: [u8;4096],
    buflen:  usize,
    bufpos:  usize,
    hashpos: usize,

    blocksize: usize,
}

impl <R,C,H,N> Chunker<R,C,H,N> where
R: Read,
C: rollsum::Engine<Digest=u32> + Default,
N: generic_array::ArrayLength<u8>,
H: sha2::Digest<OutputSize=N> + Default
{
    fn new(io: R) -> Chunker<R,C,H,N> {
        Chunker{
            io: io,
            chunker: C::default(),
            hasher: H::default(),
            buf: [0;4096],
            buflen: 0,
            bufpos: 0,
            hashpos: 0,
            blocksize: 0,
        }
    }
    fn fill(&mut self) -> bool {
        match self.io.read(&mut self.buf) {
            Err(e) => panic!(e),
            Ok(some) => {
                if some < 1 {
                    return false;
                } else {
                    self.buflen  = some;
                    self.bufpos  = 0;
                    self.hashpos = 0;
                    return true;
                }
            }
        }
    }
}

pub struct Chunk {
    pub hash: String,
    pub size: usize,
}

impl<R,C,H,N> Iterator for Chunker<R,C,H,N> where
R: Read,
C: rollsum::Engine<Digest=u32> + Default,
N: generic_array::ArrayLength<u8>,
H: sha2::Digest<OutputSize=N> + Default,
{
    type Item = Chunk;
    fn next(&mut self) -> Option<Self::Item> {
        let chunk_mask = (1 << 11) - 1;

        loop {
            if self.bufpos >= self.buflen {
                self.hasher.input(&self.buf[self.hashpos..self.buflen]);

                if !self.fill() {
                    let rest = self.buflen - self.hashpos;
                    if rest  > 0 {
                        let hash = std::mem::replace(&mut self.hasher, H::default()).result();
                        let r = Chunk{
                            hash:   format!("{}", hash.to_hex()),
                            size:   self.blocksize,
                        };
                        self.bufpos  = 0;
                        self.buflen  = 0;
                        self.hashpos = 0;
                        return Some(r);
                    }
                    return None;
                }

            }

            self.chunker.roll_byte(self.buf[self.bufpos]);
            self.bufpos += 1;
            self.blocksize += 1;

            if self.chunker.digest() & chunk_mask == chunk_mask {
                self.hasher.input(&self.buf[self.hashpos..self.bufpos-1]);
                let hash = std::mem::replace(&mut self.hasher, H::default()).result();
                let r = Chunk{
                    hash:   format!("{}", hash.to_hex()),
                    size:   self.blocksize,
                };

                self.hashpos = self.bufpos-1;
                self.blocksize = 0;

                return Some(r);
            }
        }
    }
}



fn main() {
    let mut blocks: HashMap<String, bool> = HashMap::new();

    let mut insize      = 0;
    let mut outsize     = 0;

    for filename in env::args().skip(1) {
        let path = PathBuf::from(filename.clone());
        let mut file = File::open(path).unwrap();
        insize += file.metadata().unwrap().len();

        let chunker = Chunker::<File,::rollsum::Bup, sha2::Sha256,
        generic_array::typenum::U32>::new(file);

        for chunk in chunker {
            if let None = blocks.insert(chunk.hash.clone(), true) {
                outsize += chunk.size;
                print!("{}", "+".red())
            } else {
                print!("{}", "=".green())
            }

            println!("{} {}", chunk.hash, chunk.size);
        }

        println!("-----------------------------------------------------------------------");

    }
    println!("==============================================");
    let pc = 100.0 * (outsize as f32 / insize as f32);
    println!("files size: {} after dedup: {} ({:.2}%) blocks: {}", insize, outsize, pc, blocks.len());
}

