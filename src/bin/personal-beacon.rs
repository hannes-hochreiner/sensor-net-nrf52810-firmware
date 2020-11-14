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
// use lsm303agr::{AccelOutputDataRate, Lsm303agr};
use rtic::app;
use nrf52810_pac::generic::Variant::Val;
// mod sht3;
use common::radio;

#[app(device = nrf52810_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        radio: common::radio::Radio,
        rtc: hal::rtc::Rtc<nrf52810_pac::RTC0, hal::rtc::Started>
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device: nrf52810_pac::Peripherals = cx.device;

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

        // rtic::pend(nrf52810_pac::Interrupt::POWER_CLOCK);

        let radio = radio::Radio::new(device.RADIO);
        radio.init_transmission();

        init::LateResources {
            radio: radio,
            rtc: rtc
        }
    }

    #[task(binds = RTC0, resources = [radio, rtc])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources.rtc.get_event_triggered(hal::rtc::RtcInterrupt::Compare0, true);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        ctx.resources.radio.start_transmission();
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [radio])]
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

        match radio.state() {
            Val(nrf52810_pac::radio::state::STATE_A::DISABLED) => {
                // ctx.resources.uart.write_fmt(format_args!("DISABLED\n")).unwrap();

                // radio.init_transmission();
                // radio.start_transmission();
            },
            Val(nrf52810_pac::radio::state::STATE_A::TX) => {
            },
            Val(nrf52810_pac::radio::state::STATE_A::RX) => {

                if radio.is_ready() {
                    radio.clear_ready();
                } else if radio.is_address() {
                    radio.clear_address();
                } else if radio.is_payload() {
                    radio.clear_payload();
                }
            },
            _ => {
            }
        }
    }
};
