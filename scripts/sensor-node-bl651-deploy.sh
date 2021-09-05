#!/bin/bash

cargo build --release --bin sensor-node-bl651
gdb --batch \
  -ex "target remote | ssh hc1 \"sudo openocd -c 'gdb_port pipe; log_output /dev/null; source [find interface/raspberrypi-native.cfg]; transport select swd; source [find target/nrf52.cfg]; bcm2835gpio_swd_nums 23 24; bcm2835gpio_trst_num 18; init'\"" \
  -ex "monitor reset halt" \
  -ex "load" \
  -ex "detach" \
  -ex "quit" \
  target/thumbv7em-none-eabi/release/sensor-node-bl651
