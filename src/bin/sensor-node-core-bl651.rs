#![no_std]
#![no_main]

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
use common::p0;
use common::rng;
use common::rtc;
use common::sht4x;
use common::timer;
use common::twim;
// use embedded_hal::blocking::delay::DelayMs;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut device = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();

    let clock = clock::Clock::new(device.CLOCK);
    let mut clock = clock.start_lfclk(clock::Source::Xtal, false, false);
    // TODO: switch back to xtal
    // let mut clock = clock.start_lfclk(clock::Source::RC, false, false);

    // set up radio
    let mut radio = radio::Radio::new(device.RADIO);
    radio.set_enabled(false);

    // get device id
    let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32)
        + (device.FICR.deviceid[0].read().bits() as u64);
    let part_id = device.FICR.info.part.read().bits();

    // rng test
    // let rng = rng::Rng::new(device.RNG, &mut core.NVIC);
    // let rng = rng.start_getting_value();
    // let (rng, val) = rng.wait_for_value();
    // rng test

    // set up timer
    // let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);

    // // set up twim
    // let mut p0 = p0::P0::new(device.P0);
    // // SCL P0.25
    // p0.configure_pin(
    //     25,
    //     p0::Dir::Input,
    //     p0::Pull::PullUp,
    //     p0::Drive::S0D1,
    //     p0::Input::Connect,
    //     p0::Sense::Disabled,
    // );
    // // SDA P0.26
    // p0.configure_pin(
    //     26,
    //     p0::Dir::Input,
    //     p0::Pull::PullUp,
    //     p0::Drive::S0D1,
    //     p0::Input::Connect,
    //     p0::Sense::Disabled,
    // );

    let serial = {
        let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);
        let mut twim = twim::Twim::new(
            &mut device.TWIM0,
            &mut core.NVIC,
            &mut device.P0,
            25,
            26,
            twim::Frequency::K100,
        );
        let mut sht4x = sht4x::SHT4X::new(&mut twim, &mut timer, 0x44);
        sht4x.start_reading_serial().unwrap();
        let res = sht4x.wait_for_serial().unwrap();
        drop(sht4x);
        res
    };

    let mut rtc = rtc::Rtc::new(&mut device.RTC0, &mut core.NVIC);
    rtc.set_prescaler(3276); // 0.1 s
                             // rtc.set_compare(30); // 3 s
    rtc.set_compare(600); // 1 min

    // initialize index
    let mut index = 0u32;

    loop {
        // wait
        rtc.start();
        rtc.wait();

        // get sensor data
        let measurement = {
            let mut timer = timer::Timer::new(&mut device.TIMER0, &mut core.NVIC);
            let mut twim = twim::Twim::new(
                &mut device.TWIM0,
                &mut core.NVIC,
                &mut device.P0,
                25,
                26,
                twim::Frequency::K400,
            );
            let mut sht4x = sht4x::SHT4X::new(&mut twim, &mut timer, 0x44);

            sht4x.start_measurement().unwrap();
            sht4x.wait_for_measurement().unwrap()
        };

        // create package
        let mut package: [u8; 28] = [0; 28];

        package[0..2].copy_from_slice(&4u16.to_le_bytes()[..]);
        package[2..10].copy_from_slice(&device_id.to_le_bytes()[..]);
        package[10..14].copy_from_slice(&part_id.to_le_bytes()[..]);
        package[14..18].copy_from_slice(&index.to_le_bytes()[..]);
        package[18..20].copy_from_slice(&serial.to_le_bytes()[0..2]);
        package[20..24].copy_from_slice(&measurement.temperature.to_le_bytes()[..]);
        package[24..28].copy_from_slice(&measurement.humidity.to_le_bytes()[..]);

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

// #[interrupt]
// fn TWIM0_TWIS0_TWI0() {
//     let mut peri = unsafe {pac::Peripherals::steal()};

//     peri.TWIM0.intenclr.write(|w| w.stopped().clear());

//     let tmp = peri.TWIM0.events_stopped.read().events_stopped().is_generated();
//     let tmp2 = peri.TWIM0.events_stopped.read().bits();
//     let tmp3 = 5;
// }

// #[app(device = nrf52810_pac, peripherals = true)]
// const APP: () = {
//     struct Resources {
//         radio: radio::Radio,
//         delay: hal::delay::Delay,
//         rtc: hal::rtc::Rtc<nrf52810_pac::RTC0>,
//         i2c: hal::twim::Twim<nrf52810_pac::TWIM0>,
//         device_id: u64,
//         part_id: u32,
//         sensor_id: u32,
//         #[init(0)]
//         index: u32,
//         rng: hal::rng::Rng,
//         ccm: hal::ccm::Ccm,
//         key: [u8; 16],
//     }

//     #[init]
//     fn init(cx: init::Context) -> init::LateResources {
//         let device: nrf52810_pac::Peripherals = cx.device;
//         let core = cx.core;
//         let port0 = hal::gpio::p0::Parts::new(device.P0);
//         let mut delay = hal::delay::Delay::new(core.SYST);
//         let i2c_pins = hal::twim::Pins {
//             sda: port0.p0_15.into_floating_input().degrade(),
//             scl: port0.p0_13.into_floating_input().degrade(),
//         };
//         let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);

//         // set up SHT3
//         // let sensor_id = common::sht3::SHT3::new(&mut i2c, &mut delay)
//         //     .init()
//         //     .unwrap();
//         let sensor_id = 0x01020304u32;

//         i2c.disable();

//         // set up clocks
//         hal::clocks::Clocks::new(device.CLOCK)
//             .set_lfclk_src_external(hal::clocks::LfOscConfiguration::NoExternalNoBypass)
//             .start_lfclk()
//             .enable_ext_hfosc();

//         // set up RTC
//         let mut rtc = hal::rtc::Rtc::new(device.RTC0, 3276).unwrap(); // => 10Hz
//         rtc.set_compare(hal::rtc::RtcCompareReg::Compare0, 600)
//             .unwrap(); // => 1 min
//         rtc.enable_event(hal::rtc::RtcInterrupt::Compare0);
//         rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
//         rtc.enable_counter();

//         // set up radio
//         let mut radio = radio::Radio::new(device.RADIO);
//         radio.set_enabled(false);

//         // get device id
//         let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32)
//             + (device.FICR.deviceid[0].read().bits() as u64);
//         let part_id = device.FICR.info.part.read().bits();

//         // set up rng
//         let rng = hal::rng::Rng::new(device.RNG);

//         // set up ccm
//         let ccm = hal::ccm::Ccm::init(device.CCM, device.AAR, hal::ccm::DataRate::_2Mbit);

//         // set up power
//         let mut power = power::Power::new(device.POWER);
//         power.set_mode(power::Mode::LowPower);

//         init::LateResources {
//             radio: radio,
//             delay: delay,
//             rtc: rtc,
//             i2c: i2c,
//             device_id: device_id,
//             part_id: part_id,
//             sensor_id: sensor_id,
//             rng: rng,
//             ccm: ccm,
//             key: get_key(),
//         }
//     }

//     #[task(binds = RTC0, resources = [rtc, radio, i2c, delay, device_id, part_id, sensor_id, index, rng, ccm, key])]
//     fn rtc_handler(ctx: rtc_handler::Context) {
//         ctx.resources
//             .rtc
//             .reset_event(hal::rtc::RtcInterrupt::Compare0);
//         // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
//         ctx.resources.i2c.enable();

//         // let mut sht3 = common::sht3::SHT3::new(ctx.resources.i2c, ctx.resources.delay);
//         // let meas = sht3.get_measurement().unwrap();

//         // core::mem::drop(sht3);
//         ctx.resources.i2c.disable();

//         let mut iv = [0u8; 8];
//         ctx.resources.rng.random(&mut iv);
//         let mut ccm_data = hal::ccm::CcmData::new(*ctx.resources.key, iv);

//         // assemble encryption package
//         let mut enc_pac = [0u8; 29];

//         enc_pac[1] = 26;
//         copy_into_array(&ctx.resources.device_id.to_le_bytes(), &mut enc_pac[3..11]);
//         copy_into_array(&ctx.resources.part_id.to_le_bytes(), &mut enc_pac[11..15]);
//         copy_into_array(&ctx.resources.index.to_le_bytes(), &mut enc_pac[15..19]);
//         copy_into_array(&ctx.resources.sensor_id.to_le_bytes(), &mut enc_pac[19..21]);
//         // copy_into_array(&meas.temperature.to_le_bytes(), &mut enc_pac[21..25]);
//         // copy_into_array(&meas.humidity.to_le_bytes(), &mut enc_pac[25..29]);

//         let mut enc_pac_enc = [0u8; 33];
//         let mut scratch = [0u8; 43];

//         ctx.resources
//             .ccm
//             .encrypt_packet(&mut ccm_data, &enc_pac, &mut enc_pac_enc, &mut scratch)
//             .unwrap();

//         let data: &[&[u8]] = &[&0x8005u16.to_le_bytes(), &iv, &enc_pac_enc[3..33]];

//         *ctx.resources.index += 1;
//         ctx.resources.radio.init_transmission();
//         ctx.resources.radio.start_transmission(data);
//         ctx.resources.rtc.clear_counter();
//         // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
//     }

//     #[task(binds = RADIO, resources = [radio])]
//     fn radio_handler(ctx: radio_handler::Context) {
//         let radio = ctx.resources.radio;
//         let _event_disabled = radio.event_disabled();

//         radio.event_reset_all();
//         radio.set_enabled(false);
//     }
// };
