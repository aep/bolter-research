bolter
======

this is a research project aimed to improve content addressable storage  of statically linked position
independant executables.

We'd like to be able to store multiple statically linked binaries in a space efficient way without
knowing all the versions upfront, so neither regular compression nor the classic multicall binary will work.

"testbin" contains a hyper client/server with separate binaries for client and server, as well as a multicall.
They share large parts of library code. Some won't be shared because it's not used.
For example the linker strips hyper server stuff when linking the client.


|          | client  | server  | total | multicall   |
|----------|---------|---------|-------|-------------|
| debug    | 21M     | 19M     | 40M   | 22M  (55%)  |
| release  | 4.8M    | 4.7M    | 9.5M  | 5M   (52%)  |
| stripped | 1.4M    | 1.3M    | 2.7M  | 1.6M (59%)  |

multicall is obviously very efficient. we'll have to be pretty clever to get even close to that.
The stripped binary looks abit worse here on paper because the sections where sharing
is most efficient have been removed (such as string tables, debug things).
Since we only really care about the efficiency of a binary that goes on a real device,
the rest of the tests are always stripped as much as possible,

testing dedup algos:

| alogrithm | compression | shards |
|-----------|-------------|--------|
|bup(15)    | 94.89%      | 595    |
|bup(13)    | 90.18%      | 2034   |
|bup(11)    | 85.13%      | 7320   |
|bup(7)     | 77.37%      | 104105 |
|elf        | 95.95%      | 8250   |


rolling hashes such as bup are inefficient because all library code is relocated.
Even if both binaries use the same library, the code will not match because the addresses are different.

the "elf" algo splits by elf sections and symbols from .symtab,
which are a pretty good indication of where code will be identical.
But the code was relocated so those markers aren't helping.

in ~/.cargo/config
```
[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "link-arg=-Wl,--emit-relocs",
]
```

will emit relocation symbols in the final executable so we can see where relocations have been
applied and undo them with:

```
cargo run --bin dislocate ./testbin/target/release/client ./testbin/target/release/server
```

| alogrithm | compression | shards |
|-----------|-------------|--------|
|bup(15)    | 88.77%      | 126    |
|bup(13)    | 88.31%      | 460    |
|bup(11)    | 86.08%      | 1831   |
|bup(7)     | 79.82%      | 28500  |
|elf        | 80.24%      | 4336   |


elf is already outperforming bup, but still significantly worse than multicall


