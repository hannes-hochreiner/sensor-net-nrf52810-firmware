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
use common::radio;

#[app(device = nrf52810_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        radio: common::radio::Radio,
        rtc: hal::rtc::Rtc<nrf52810_pac::RTC0, hal::rtc::Started>,
        device_id: u64,
        part_id: u32,
        #[init(0)]
        index: u32
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

        // get device id
        let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32) + (device.FICR.deviceid[0].read().bits() as u64);
        let part_id = device.FICR.info.part.read().bits();
        
        // rtic::pend(nrf52810_pac::Interrupt::POWER_CLOCK);

        let radio = radio::Radio::new(device.RADIO);
        radio.init_transmission();

        init::LateResources {
            radio: radio,
            rtc: rtc,
            device_id: device_id,
            part_id: part_id
        }
    }

    #[task(binds = RTC0, resources = [radio, rtc, device_id, part_id, index])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources.rtc.get_event_triggered(hal::rtc::RtcInterrupt::Compare0, true);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);

        let sensor_id: u16 = 0xABCD;
        let acc_x: f32 = 3.15;
        let acc_y: f32 = 3.15;
        let acc_z: f32 = 3.15;
        let mag_x: f32 = 3.15;
        let mag_y: f32 = 3.15;
        let mag_z: f32 = 3.15;

        // payload: type (u8) | device_id (u64) | part_id (u32) | index (u32) | sensor_id (u16) | acc_x (f32) | acc_y (f32) | acc_z (f32) | mag_x (f32) | mag_y (f32) | mag_z (f32)
        // size (u8) + 43 bytes = 44 bytes
        
        let data: &[&[u8]] = &[
            &[3u8], 
            &ctx.resources.device_id.to_le_bytes(),
            &ctx.resources.part_id.to_le_bytes(),
            &ctx.resources.index.to_le_bytes(),
            &sensor_id.to_le_bytes(),
            &acc_x.to_le_bytes(),
            &acc_y.to_le_bytes(),
            &acc_z.to_le_bytes(),
            &mag_x.to_le_bytes(),
            &mag_y.to_le_bytes(),
            &mag_z.to_le_bytes(),
        ];

        *ctx.resources.index += 1;

        ctx.resources.radio.start_transmission(data);
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [radio])]
    fn radio_handler(ctx: radio_handler::Context) {
        let radio = ctx.resources.radio;

        radio.clear_all();

        // let _event_ready = radio.event_ready();
        // let _event_address = radio.event_address();
        // let _event_payload = radio.event_payload();
        // let _event_end = radio.event_end();
        let _event_disabled = radio.event_disabled();
        // let _event_devmatch = radio.event_devmatch();
        // let _event_devmiss = radio.event_devmiss();
        // let _event_rssiend = radio.event_rssiend();
        // let _event_bcmatch = radio.event_bcmatch();
        // let _event_crcok = radio.event_crcok();
        // let _event_crcerror = radio.event_crcerror();
        
        radio.event_reset_all();
    }
};
