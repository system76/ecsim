#!/usr/bin/env bash

set -ex

rm -rf banks
mkdir -p banks
cd banks

for bank in 0 1 2
do
    dd if=../ec.rom of=bank$bank.bin bs=32768 count=1
    dd if=../ec.rom of=bank$bank.bin bs=32768 count=1 conv=notrunc seek=1 skip=$(($bank + 1))
    d52 -bdnpt bank$bank.bin
    expand bank$bank.d52 > bank$bank.d52.expanded
    mv bank$bank.d52.expanded bank$bank.d52
done
