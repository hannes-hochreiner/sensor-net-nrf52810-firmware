#![no_std]
#![no_main]

use core::mem;

use common::sht4x::Measurement;
use embedded_hal::adc;
use hal::prelude::OutputPin;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

// use cortex_m::asm;
// use cortex_m_rt::entry;
use nrf52810_hal as hal;
use nrf52810_pac as pac;
use pac::interrupt;
// use nrf52810_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
// use rtic::app;
// use common::sht3;
use common::power;
use common::radio;
use common::utils::{copy_into_array, get_key};
// use embedded_hal::blocking::{i2c as i2c, delay as delay};
use common::clock;
use common::mmc5603nj;
use common::p0;
use common::rng;
use common::rtc;
use common::saadc;
use common::sht4x;
use common::timer;
use common::twim;
// use embedded_hal::blocking::delay::DelayMs;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut device = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();

    // read the configuration
    let conf0 = device.UICR.customer[0].read().bits();
    let conf_board_type = (conf0 >> 24) as u8;
    let conf_version_major = (conf0 >> 16) as u8;
    let conf_version_minor = (conf0 >> 8) as u8;
    let conf_version_patch = conf0 as u8;

    let clock = clock::Clock::new(device.CLOCK);
    let mut clock = clock.start_lfclk(clock::Source::Xtal, false, false); // TODO: switch back to xtal
    // let mut clock = clock.start_lfclk(clock::Source::RC, false, false);

    // set up radio
    let mut radio = radio::Radio::new(device.RADIO);
    radio.set_enabled(false);

    // get device id
    let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32)
        + (device.FICR.deviceid[0].read().bits() as u64);
    let part_id = device.FICR.info.part.read().bits();

    // adc test
    // let mut adc_result = 0u32;
    // let adc_result = 0u16;
    // // select P0.03/AIN1 as the positive input
    // device.P0.pin_cnf[3].write(|w| w.input().connect());
    // device.SAADC.ch[0].pselp.write(|w| w.pselp().analog_input1());
    // // device.SAADC.ch[0].pselp.write(|w| w.pselp().vdd());
    // // set gain 1
    // device.SAADC.ch[0].config.write(|w| w.gain().gain1());
    // // device.SAADC.ch[0].config.write(|w| w.gain().gain1_6());
    // // set result pointer
    // device.SAADC.result.ptr.write(|w| unsafe { w.ptr().bits((&adc_result as *const u16) as u32) });
    // // set max count 1
    // device.SAADC.result.maxcnt.write(|w| unsafe { w.maxcnt().bits(1) });
    // // enable
    // device.SAADC.enable.write(|w| w.enable().enabled());

    // let mut adc_ready = false;
    // let mut adc_cntr = 0;

    // while !adc_ready {
    //     adc_ready = device.SAADC.status.read().status().is_ready();
    //     adc_cntr += 1;
    // }
    // adc test

    // loop {
    //     let mut saadc = saadc::Saadc::new(device.SAADC, device.P0);
    //     let battery_voltage = saadc.getValue();
    //     let tmp = saadc.free();
    //     device.SAADC = tmp.0;
    //     device.P0 = tmp.1;
    //     // start ADC
    //     device.SAADC.tasks_start.write(|w| w.tasks_start().trigger());
    //     while device.SAADC.events_started.read().events_started().is_not_generated() {}

    //     // trigger sample task
    //     device.SAADC.tasks_sample.write(|w| w.tasks_sample().trigger());
    //     while device.SAADC.events_end.read().events_end().is_not_generated() {}

    //     let res = adc_result as f32 * 0.6 / 1024.0 / 0.4;
    //     // Chevron: 1023 => 1.49853528 2021-03-01 20:26
    //     // Premio: 848 => 1.2421875 2021-03-01 20:32

    //     device.SAADC.events_started.write(|w| w.events_started().not_generated());
    //     device.SAADC.events_end.write(|w| w.events_end().not_generated());
    // }

    // set up timer
    // let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);

    // // set up twim
    // let mut p0 = p0::P0::new(device.P0);
    // // SCL P0.23
    // p0.configure_pin(
    //     23,
    //     p0::Dir::Input,
    //     p0::Pull::PullUp,
    //     p0::Drive::S0D1,
    //     p0::Input::Connect,
    //     p0::Sense::Disabled,
    // );
    // // SDA P0.22
    // p0.configure_pin(
    //     22,
    //     p0::Dir::Input,
    //     p0::Pull::PullUp,
    //     p0::Drive::S0D1,
    //     p0::Input::Connect,
    //     p0::Sense::Disabled,
    // );

    // let serial = 0u32;
    let serial = {
        let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);
        let mut twim = twim::Twim::new(
            &mut device.TWIM0,
            &mut core.NVIC,
            &mut device.P0,
            23,
            22,
            twim::Frequency::K400,
        );
        let mut sht4x = sht4x::SHT4X::new(&mut twim, &mut timer, 0x44);
        sht4x.start_reading_serial().unwrap();
        sht4x.wait_for_serial().unwrap()
    };

    let mut rtc = rtc::Rtc::new(&mut device.RTC0, &mut core.NVIC);
    rtc.set_prescaler(3276); // 0.1 s

    if cfg!(debug_assertions) {
        rtc.set_compare(30); // 3 s
    } else {
        rtc.set_compare(600); // 1 min
    }

    // initialize index
    let mut index = 0u32;

    loop {
        // wait
        rtc.start();
        rtc.wait();

        // get battery voltage
        let mut saadc = saadc::Saadc::new(device.SAADC, device.P0);
        let battery_voltage = saadc.getValue();
        // let battery_voltage = 1f32;
        let tmp = saadc.free();
        device.SAADC = tmp.0;
        device.P0 = tmp.1;

        // if battery voltage is lower 1.1 V and we are not running in debug mode,
        // go into a sleep loop
        if battery_voltage < 1.1 && !cfg!(debug_assertions) {
            // let port0 = hal::gpio::p0::Parts::new(device.P0);
            // let mut p19 = port0.p0_19.into_push_pull_output(hal::gpio::Level::High);
            // p19.set_high().unwrap();
            // device.P0.pin_cnf[19].write(|w| w.dir().output());
            // device.P0.out.write(|w| w.pin19().high());
            // device.P0.outset.write(|w| w.pin19().set());
            loop {
                rtc.start();
                rtc.wait();
            }
        }

        // get sensor data
        // let mut temperature = 25f32;
        // let mut humidity = 50f32;

        // sht4x_measurement(&mut twim, &mut timer);

        // let measurement = Measurement {
        //     temperature: 25.0,
        //     humidity: 50.5,
        // };

        let measurement = {
            let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);
            let mut twim = twim::Twim::new(
                &mut device.TWIM0,
                &mut core.NVIC,
                &mut device.P0,
                23,
                22,
                twim::Frequency::K400,
            );
            let mut sht4x = sht4x::SHT4X::new(&mut twim, &mut timer, 0x44);
            sht4x.start_measurement().unwrap();
            sht4x.wait_for_measurement().unwrap()
        };

        // let mag = {
        //     let mut twim = twim::Twim::new(
        //         &mut device.TWIM0,
        //         &mut core.NVIC,
        //         &mut device.P0,
        //         23,
        //         22,
        //         twim::Frequency::K400,
        //     );
        //     let mut mmc = mmc5603nj::MMC5603NJ::new(&mut twim, 0b00110000);
        //     // mmc.start_magnetic__measruement(mmc5603nj::Mmc5603njBias::Set).unwrap();
        //     // let meas1 = mmc.wait_for_magnetic_measurement().unwrap();
        //     // mmc.start_magnetic__measruement(mmc5603nj::Mmc5603njBias::Reset).unwrap();
        //     // let meas2 = mmc.wait_for_magnetic_measurement().unwrap();

        //     // ((meas1.0 - meas2.0) / 2f32, (meas1.1 - meas2.1) / 2f32, (meas1.2 - meas2.2) / 2f32)
        //     mmc.start_magnetic__measruement(mmc5603nj::Mmc5603njBias::Reset).unwrap();
        //     mmc.wait_for_magnetic_measurement().unwrap()
        // };

        // create package
        let mut package: [u8; 54] = [0; 54];

        package[0..2].copy_from_slice(&5u16.to_le_bytes()[..]);
        package[2..10].copy_from_slice(&device_id.to_le_bytes()[..]);
        package[10..14].copy_from_slice(&part_id.to_le_bytes()[..]);
        package[14..18].copy_from_slice(&index.to_le_bytes()[..]);
        package[18..22].copy_from_slice(&0u32.to_le_bytes()[..]);
        package[22..24].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[24..26].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[26..28].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[28..32].copy_from_slice(&0u32.to_le_bytes()[..]);
        package[32..34].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[34..36].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[36..38].copy_from_slice(&0i16.to_le_bytes()[..]);
        package[38..42].copy_from_slice(&serial.to_le_bytes()[..]);
        package[42..46].copy_from_slice(&measurement.temperature.to_le_bytes()[..]);
        package[46..50].copy_from_slice(&measurement.humidity.to_le_bytes()[..]);
        package[50..54].copy_from_slice(&battery_voltage.to_le_bytes()[..]);

        // increment index
        index += 1;

        // send package
        let clock_hf_active = clock.start_hfclk();

        radio.init_transmission();
        let package_wrapper: [&[u8]; 1] = [&package];
        radio.start_transmission(&package_wrapper);

        let peri = unsafe { pac::Peripherals::steal() };

        while peri.RADIO.events_end.read().events_end().is_not_generated() {}

        radio.event_reset_all();
        radio.set_enabled(false);

        clock = clock_hf_active.stop_hfclk();
    }
}
