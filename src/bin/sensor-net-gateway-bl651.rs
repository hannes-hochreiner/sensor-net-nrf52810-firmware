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
        i2c: hal::twim::Twim<nrf52810_pac::TWIM0>
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device: nrf52810_pac::Peripherals = cx.device;
        let core = cx.core;
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let mut led_green = port0.p0_24.into_push_pull_output(Level::Low);
        led_green.set_high().unwrap();
        let pins = hal::uarte::Pins {
            rxd: port0.p0_08.into_floating_input().degrade(),
            txd: port0.p0_06.into_push_pull_output(Level::Low).degrade(),
            cts: None,
            rts: None
        };
        let uart = hal::uarte::Uarte::new(device.UARTE0, pins, hal::uarte::Parity::EXCLUDED, hal::uarte::Baudrate::BAUD1M);
        
        let mut delay = hal::delay::Delay::new(core.SYST);
        
        let i2c_pins = hal::twim::Pins {
            sda: port0.p0_15.into_floating_input().degrade(),
            scl: port0.p0_13.into_floating_input().degrade(),
        };
        let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);
        
        // set up SHT3
        common::sht3::SHT3::new(&mut i2c, &mut delay).init().unwrap();

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

        init::LateResources {
            uart: uart,
            radio: radio,
            delay: delay,
            rtc: rtc,
            i2c: i2c
        }
    }

    #[task(binds = RTC0, resources = [uart, rtc, i2c, delay])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources.rtc.get_event_triggered(hal::rtc::RtcInterrupt::Compare0, true);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        let mut sht3 = common::sht3::SHT3::new(ctx.resources.i2c, ctx.resources.delay);
        let meas = sht3.get_measurement().unwrap();
        ctx.resources.uart.write_fmt(format_args!("{{\"\temperature\": {}, \"humidity\": {}}}\n", meas.temperature, meas.humidity)).unwrap();
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [uart, radio, delay])]
    fn radio_handler(ctx: radio_handler::Context) {
        let radio = ctx.resources.radio;

        // ctx.resources.uart.write_fmt(format_args!("radio handler called\n")).unwrap();

        radio.clear_all();

        let _event_ready = radio.event_ready();
        let _event_address = radio.event_address();
        let _event_payload = radio.event_payload();
        let _event_end = radio.event_end();
        let _event_disabled = radio.event_disabled();
        let _event_devmatch = radio.event_devmatch();
        let _event_devmiss = radio.event_devmiss();
        let _event_rssiend = radio.event_rssiend();
        let _event_bcmatch = radio.event_bcmatch();
        let _event_crcok = radio.event_crcok();
        let _event_crcerror = radio.event_crcerror();
        
        radio.event_reset_all();

        if !_event_ready {
            // radio.init_reception();
            // radio.start_reception();
        } else {
            if _event_rssiend {
                ctx.resources.uart.write_fmt(format_args!("RSSI: -{}dB\n", radio.rssi())).unwrap();
            }
            if _event_payload {
                ctx.resources.uart.write_fmt(format_args!("payload: {:?}\n", radio.payload())).unwrap();
            }   
            if _event_disabled {
                radio.start_reception();
            }
        }
    }
};
