#!/usr/bin/env bash

set -e

rm -rf gpio
mkdir -p gpio

cargo build --release --no-default-features --features debug_xram
target/release/ecsim 2>&1 | tee gpio/gpio.log
grep ' (GPIO ' gpio/gpio.log |
grep ' store ' |
cut -d ' ' -f 5,9 |
sed 's/)/ =/' | sed 's/]/;/' | tee gpio/gpio.h

echo "Results saved in gpio/gpio.h"
