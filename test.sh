#!/bin/sh


mkdir -p ./testbin/target/release/workme/
cp ./testbin/target/release/client ./testbin/target/release/server ./testbin/target/release/workme/

for i in ./testbin/target/release/workme/client ./testbin/target/release/workme/server
do
    objcopy $i --only-keep-debug $i.d
    strip $i
    cargo run --bin dislocate $i $i.d $i.locations

    #doing this correctly later..
    cat $i.locations >> $i
done



