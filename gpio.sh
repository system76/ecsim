#!/usr/bin/env bash

set -e

rm -rf gpio
mkdir -p gpio

cargo build --release --no-default-features --features debug_xram
target/release/ecsim 2>&1 | tee gpio/gpio.log
grep ' (GPIO ' gpio/gpio.log |
grep ' store ' |
cut -d ' ' -f 5,9 |
sed 's/)/ =/' | sed 's/]/;/' |
sed 's/\(GPCR.*\) = 0x00;/\1 = GPIO_ALT;/' |
sed 's/\(GPCR.*\) = 0x02;/\1 = GPIO_ALT | GPIO_DOWN;/' |
sed 's/\(GPCR.*\) = 0x04;/\1 = GPIO_ALT | GPIO_UP;/' |
sed 's/\(GPCR.*\) = 0x40;/\1 = GPIO_OUT;/' |
sed 's/\(GPCR.*\) = 0x42;/\1 = GPIO_OUT | GPIO_DOWN;/' |
sed 's/\(GPCR.*\) = 0x44;/\1 = GPIO_OUT | GPIO_UP;/' |
sed 's/\(GPCR.*\) = 0x80;/\1 = GPIO_IN;/' |
sed 's/\(GPCR.*\) = 0x82;/\1 = GPIO_IN | GPIO_DOWN;/' |
sed 's/\(GPCR.*\) = 0x84;/\1 = GPIO_IN | GPIO_UP;/' |
tee gpio/gpio.h

echo "Results saved in gpio/gpio.h"
