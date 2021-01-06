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
// use nrf52810_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use rtic::app;
// use common::sht3;
use common::radio;
// use embedded_hal::blocking::{i2c as i2c, delay as delay};

#[app(device = nrf52810_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        radio: radio::Radio,
        delay: hal::delay::Delay,
        rtc: hal::rtc::Rtc<nrf52810_pac::RTC0>,
        i2c: hal::twim::Twim<nrf52810_pac::TWIM0>,
        device_id: u64,
        part_id: u32,
        sensor_id: u16,
        #[init(0)]
        index: u32,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device: nrf52810_pac::Peripherals = cx.device;
        let core = cx.core;
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let mut delay = hal::delay::Delay::new(core.SYST);
        let i2c_pins = hal::twim::Pins {
            sda: port0.p0_15.into_floating_input().degrade(),
            scl: port0.p0_13.into_floating_input().degrade(),
        };
        let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);

        // set up SHT3
        let sensor_id = common::sht3::SHT3::new(&mut i2c, &mut delay)
            .init()
            .unwrap();

        // set up clocks
        hal::clocks::Clocks::new(device.CLOCK)
            .set_lfclk_src_rc()
            .start_lfclk()
            .enable_ext_hfosc();

        // set up RTC
        let mut rtc = hal::rtc::Rtc::new(device.RTC0, 3276).unwrap(); // => 10Hz
        rtc.set_compare(hal::rtc::RtcCompareReg::Compare0, 600)
            .unwrap(); // => 1 min
        rtc.enable_event(hal::rtc::RtcInterrupt::Compare0);
        rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        rtc.enable_counter();

        // set up radio
        let radio = radio::Radio::new(device.RADIO);
        radio.init_reception();
        radio.start_reception();

        // get device id
        let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32)
            + (device.FICR.deviceid[0].read().bits() as u64);
        let part_id = device.FICR.info.part.read().bits();

        init::LateResources {
            radio: radio,
            delay: delay,
            rtc: rtc,
            i2c: i2c,
            device_id: device_id,
            part_id: part_id,
            sensor_id: sensor_id,
        }
    }

    #[task(binds = RTC0, resources = [rtc, i2c, delay, device_id, part_id, sensor_id, index])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources
            .rtc
            .reset_event(hal::rtc::RtcInterrupt::Compare0);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        let mut sht3 = common::sht3::SHT3::new(ctx.resources.i2c, ctx.resources.delay);
        let meas = sht3.get_measurement().unwrap();
        // ctx.resources.uart.write_fmt(format_args!("{{\"type\":\"gateway-bl651-sensor\",\"message\":{{\"mcuId\":\"{:0>8x}-{:0>16x}\",\"index\":{},\"sensorId\":\"{:0>4x}\",\"temperature\":{},\"humidity\":{}}}}}\n", ctx.resources.part_id, ctx.resources.device_id, ctx.resources.index, ctx.resources.sensor_id, meas.temperature, meas.humidity)).unwrap();
        *ctx.resources.index += 1;
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [radio])]
    fn radio_handler(ctx: radio_handler::Context) {
        let radio = ctx.resources.radio;

        radio.clear_all();

        // let event_address = radio.event_address();
        // let event_payload = radio.event_payload();
        // let event_end = radio.event_end();
        // let event_disabled = radio.event_disabled();
        // let event_rssiend = radio.event_rssiend();
        // let event_crcok = radio.event_crcok();

        // radio.event_reset_all();

        // if event_address && event_payload && event_end && event_crcok && event_rssiend {
        //     if let Some(data) = radio.payload() {
        //         ctx.resources.led_red.set_high().unwrap();
        //         ctx.resources
        //             .uart
        //             .write_fmt(format_args!(
        //                 "{{\
        //             \"type\": \"gateway-bl651-radio\",\
        //             \"rssi\": -{},\
        //             \"data\": \"",
        //                 radio.rssi()
        //             ))
        //             .unwrap();

        //         for byte in data {
        //             ctx.resources
        //                 .uart
        //                 .write_fmt(format_args!("{:0>2x}", byte))
        //                 .unwrap();
        //         }

        //         ctx.resources
        //             .uart
        //             .write_fmt(format_args!("\"}}\n"))
        //             .unwrap();
        //         // ctx.resources.uart.write_fmt(format_args!("payload: {:?}\n", radio.payload())).unwrap();
        //         ctx.resources.led_red.set_low().unwrap();
        //     }
        // }

        // if event_disabled {
        //     radio.start_reception();
        // }
    }
};
