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
use nrf52810_hal::gpio::Level;
use nrf52810_hal::prelude::OutputPin;
// use nrf52810_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use core::fmt::Write;
use core::format_args;
use rtic::app;
// use common::sht3;
use common::radio;
// use embedded_hal::blocking::{i2c as i2c, delay as delay};

#[app(device = nrf52810_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        uart: hal::uarte::Uarte<nrf52810_hal::pac::UARTE0>,
        radio: radio::Radio,
        delay: hal::delay::Delay,
        rtc: hal::rtc::Rtc<nrf52810_pac::RTC0, hal::rtc::Started>,
        i2c: hal::twim::Twim<nrf52810_pac::TWIM0>,
        device_id: u64,
        part_id: u32,
        sensor_id: u16,
        #[init(0)]
        index: u32,
        led_green: nrf52810_hal::gpio::p0::P0_24<nrf52810_hal::gpio::Output<nrf52810_hal::gpio::PushPull>>,
        led_red: nrf52810_hal::gpio::p0::P0_23<nrf52810_hal::gpio::Output<nrf52810_hal::gpio::PushPull>>
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device: nrf52810_pac::Peripherals = cx.device;
        let core = cx.core;
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let led_green = port0.p0_24.into_push_pull_output(Level::Low);
        let led_red = port0.p0_23.into_push_pull_output(Level::Low);
        let pins = hal::uarte::Pins {
            rxd: port0.p0_08.into_floating_input().degrade(),
            txd: port0.p0_06.into_push_pull_output(Level::Low).degrade(),
            cts: Some(port0.p0_07.into_floating_input().degrade()),
            rts: Some(port0.p0_05.into_push_pull_output(Level::Low).degrade())
        };
        let uart = hal::uarte::Uarte::new(device.UARTE0, pins, hal::uarte::Parity::EXCLUDED, hal::uarte::Baudrate::BAUD1M);
        
        let mut delay = hal::delay::Delay::new(core.SYST);
        
        let i2c_pins = hal::twim::Pins {
            sda: port0.p0_15.into_floating_input().degrade(),
            scl: port0.p0_13.into_floating_input().degrade(),
        };
        let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);
        
        // set up SHT3
        let sensor_id = common::sht3::SHT3::new(&mut i2c, &mut delay).init().unwrap();

        // set up clocks
        hal::clocks::Clocks::new(device.CLOCK)
            .set_lfclk_src_rc()
            .start_lfclk()
            .enable_ext_hfosc();

        // set up RTC
        let mut rtc = hal::rtc::Rtc::new(device.RTC0);
        rtc.set_prescaler(3276).unwrap(); // => 10Hz
        rtc.set_compare(hal::rtc::RtcCompareReg::Compare0, 50).unwrap();
        rtc.enable_event(hal::rtc::RtcInterrupt::Compare0);
        rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        let rtc = rtc.enable_counter();

        // set up radio
        let radio = radio::Radio::new(device.RADIO);
        radio.init_reception();
        radio.start_reception();

        // get device id
        let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32) + (device.FICR.deviceid[0].read().bits() as u64);
        let part_id = device.FICR.info.part.read().bits();

        init::LateResources {
            uart: uart,
            radio: radio,
            delay: delay,
            rtc: rtc,
            i2c: i2c,
            device_id: device_id,
            part_id: part_id,
            sensor_id: sensor_id,
            led_green: led_green,
            led_red: led_red
        }
    }

    #[task(binds = RTC0, resources = [uart, rtc, i2c, delay, device_id, part_id, sensor_id, index, led_green])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources.rtc.get_event_triggered(hal::rtc::RtcInterrupt::Compare0, true);
        ctx.resources.led_green.set_high().unwrap();
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        let mut sht3 = common::sht3::SHT3::new(ctx.resources.i2c, ctx.resources.delay);
        let meas = sht3.get_measurement().unwrap();
        ctx.resources.uart.write_fmt(format_args!("{{\"type\":\"gateway-bl651-sensor\",\"message\":{{\"mcuId\":\"{:0>8x}-{:0>16x}\",\"index\":{},\"sensorId\":\"{:0>4x}\",\"temperature\":{},\"humidity\":{}}}}}\n", ctx.resources.part_id, ctx.resources.device_id, ctx.resources.index, ctx.resources.sensor_id, meas.temperature, meas.humidity)).unwrap();
        *ctx.resources.index += 1;
        ctx.resources.led_green.set_low().unwrap();
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [uart, radio, led_red])]
    fn radio_handler(ctx: radio_handler::Context) {
        let radio = ctx.resources.radio;

        radio.clear_all();

        let event_address = radio.event_address();
        let event_payload = radio.event_payload();
        let event_end = radio.event_end();
        let event_disabled = radio.event_disabled();
        let event_rssiend = radio.event_rssiend();
        let event_crcok = radio.event_crcok();
        
        radio.event_reset_all();

        if event_address && event_payload && event_end && event_crcok && event_rssiend {
            if let Some(data) = radio.payload() {
                ctx.resources.led_red.set_high().unwrap();
                ctx.resources.uart.write_fmt(format_args!("{{\
                    \"type\": \"gateway-bl651-radio\",\
                    \"rssi\": -{},\
                    \"data\": \"", radio.rssi())).unwrap();

                for byte in data {
                    ctx.resources.uart.write_fmt(format_args!("{:0>4x}", byte)).unwrap();
                }
                
                ctx.resources.uart.write_fmt(format_args!("\"}}\n")).unwrap();
                // ctx.resources.uart.write_fmt(format_args!("payload: {:?}\n", radio.payload())).unwrap();
                ctx.resources.led_red.set_low().unwrap();
            }
        }

        if event_disabled {
            radio.start_reception();
        }
    }
};
