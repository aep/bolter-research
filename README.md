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
The stripped binary is what will end up on a target, so 59% is our benchmark.

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


elf is already outperforming bup, but still significantly worse than multicall.
bup is only really increasing in efficiency because we have to carry over the relocation info for dislocate
to work and the symtab for elf. those compress well with rolling hash, but we dont care about those in our
final storage.
Hence, next step is moving to a model where debug symbols are separated from the binary we want to store.
see test.sh

```
objcopy client --only-keep-debug client.d
strip client
cargo run --bin dislocate client
```

| alogrithm | compression | shards |
|-----------|-------------|--------|
|bup(15)    | 95.61%      | 67     |
|bup(13)    | 91.71%      | 240    |
|bup(11)    | 90.09%      | 875    |
|bup(7)     | 86.94%      | 13015  |
|elf        | 60.14%      | 4361   |


Elf looks really impressive, and this would be a win... if the compressed output would actually work.
Dislocate removes the relocations but doesn't store them yet.
just quickly slapping the location info to the end of the binary yields:

|-----------|-------------|--------|
|bup(11)    | 79.48%      | 1156   |
|elf        | 70.29%      | 4316   |

reasonably good, but still too far off from our 59%

