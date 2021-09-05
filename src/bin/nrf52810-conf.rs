#![no_std]
#![no_main]

use nrf52810_pac as pac;
use panic_halt as _;

// An example on how to write to the user information configuration section in C can be found at
// https://infocenter.nordicsemi.com/index.jsp?topic=%2Fcom.nordic.infocenter.sdk5.v11.0.0%2Fuicr_config_example.html
//
// const uint32_t UICR_ADDR_0x80 __attribute__((at(0x10001080))) __attribute__((used)) = 0x12345678;
//
// Running this program will write the value of the static variable into the first UICR.
// Defining additional variables will write the value in the subsequent registers.
// Values can only be written as words (4 bytes).
//
// To check the value before writing it to the MCU, the following command can be used:
//
// cargo build --bin nrf52810-conf
// objdump -s target/thumbv7em-none-eabi/debug/nrf52810-conf > dump
//
// The expected output is (for a value of 0x01010100 corresponding to board type 01, version 1.1.0):
//
// 
// target/thumbv7em-none-eabi/debug/nrf52810-conf:     file format elf32-little
//
// Contents of section .conf:
//  10001080 00010101                             ....            
// Contents of section .vector_table:
//  0000 00600020 b9000000 eb020000 b9070000  .`. ............
//  0010 eb020000 eb020000 eb020000 00000000  ................
//
// The program will read the newly written value.
// Using the command "p/x _val" in the debug console, will display the value in hex.
// 
#[used]
#[link_section = ".conf"]
static UICR_ADDR_0X80: u32 = 0x01010100;

#[cortex_m_rt::entry]
fn main() -> ! {
    let device = pac::Peripherals::take().unwrap();
    let _val = device.UICR.customer[0].read().bits();

    loop {}
}
