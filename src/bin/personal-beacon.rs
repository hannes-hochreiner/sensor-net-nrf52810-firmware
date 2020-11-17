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
        index: u32,
        i2c: hal::twim::Twim<nrf52810_pac::TWIM0>,
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

        // set up sensor
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let i2c_pins = hal::twim::Pins {
            sda: port0.p0_26.into_floating_input().degrade(),
            scl: port0.p0_27.into_floating_input().degrade(),
        };
        let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);
        common::lsm303agr::LSM303AGR::new(&mut i2c).init().unwrap();

        // set up radio
        let radio = radio::Radio::new(device.RADIO);
        radio.power_off();

        init::LateResources {
            radio: radio,
            rtc: rtc,
            device_id: device_id,
            part_id: part_id,
            i2c: i2c
        }
    }

    #[task(binds = RTC0, resources = [radio, rtc, device_id, part_id, index, i2c])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources.rtc.get_event_triggered(hal::rtc::RtcInterrupt::Compare0, true);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);

        let mut sensor = common::lsm303agr::LSM303AGR::new(ctx.resources.i2c);
        let meas = sensor.get_measurement().unwrap();
        let sensor_id: u16 = 0xABCD;
        let data: &[&[u8]] = &[
            &[3u8], 
            &ctx.resources.device_id.to_le_bytes(),
            &ctx.resources.part_id.to_le_bytes(),
            &ctx.resources.index.to_le_bytes(),
            &sensor_id.to_le_bytes(),
            &meas.acc_x.to_le_bytes(),
            &meas.acc_y.to_le_bytes(),
            &meas.acc_z.to_le_bytes(),
            &meas.mag_x.to_le_bytes(),
            &meas.mag_y.to_le_bytes(),
            &meas.mag_z.to_le_bytes(),
        ];

        *ctx.resources.index += 1;

        ctx.resources.radio.init_transmission();
        ctx.resources.radio.start_transmission(data);
        ctx.resources.rtc.clear_counter();
        // ctx.resources.rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
    }

    #[task(binds = RADIO, resources = [radio])]
    fn radio_handler(ctx: radio_handler::Context) {
        let radio = ctx.resources.radio;
        let _event_disabled = radio.event_disabled();

        radio.event_reset_all();
        radio.power_off();
    }
};
