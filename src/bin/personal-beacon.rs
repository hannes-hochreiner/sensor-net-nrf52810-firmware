#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

// use cortex_m::asm;
// use cortex_m_rt::entry;
use common::power;
use common::radio;
use nrf52810_hal as hal;
use rtic::app;

#[app(device = nrf52810_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        radio: common::radio::Radio,
        rtc: hal::rtc::Rtc<nrf52810_pac::RTC0>,
        device_id: u64,
        part_id: u32,
        #[init(0)]
        index: u32,
        i2c: hal::twim::Twim<nrf52810_pac::TWIM0>,
        power: common::power::Power,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let device: nrf52810_pac::Peripherals = cx.device;

        // enable DC/DC converter
        device.POWER.dcdcen.write(|w| w.dcdcen().enabled());

        // set up clocks
        hal::clocks::Clocks::new(device.CLOCK)
            .set_lfclk_src_rc()
            .start_lfclk()
            .enable_ext_hfosc();

        // set up RTC
        let mut rtc = hal::rtc::Rtc::new(device.RTC0, 3276).unwrap();
        rtc.set_compare(hal::rtc::RtcCompareReg::Compare0, 600)
            .unwrap();
        rtc.enable_event(hal::rtc::RtcInterrupt::Compare0);
        rtc.enable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);
        rtc.enable_counter();

        // get device id
        let device_id = ((device.FICR.deviceid[1].read().bits() as u64) << 32)
            + (device.FICR.deviceid[0].read().bits() as u64);
        let part_id = device.FICR.info.part.read().bits();

        // set up sensor
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let i2c_pins = hal::twim::Pins {
            sda: port0.p0_26.into_floating_input().degrade(),
            scl: port0.p0_27.into_floating_input().degrade(),
        };
        let mut i2c = hal::twim::Twim::new(device.TWIM0, i2c_pins, hal::twim::Frequency::K400);
        common::lsm303agr::LSM303AGR::new(&mut i2c).init().unwrap();
        i2c.disable();

        // set up radio
        let mut radio = radio::Radio::new(device.RADIO);
        radio.set_enabled(false);

        // set up power
        let mut power = power::Power::new(device.POWER);
        power.set_mode(power::Mode::LowPower);

        init::LateResources {
            radio: radio,
            rtc: rtc,
            device_id: device_id,
            part_id: part_id,
            i2c: i2c,
            power: power,
        }
    }

    #[task(binds = RTC0, resources = [radio, rtc, device_id, part_id, index, i2c])]
    fn rtc_handler(ctx: rtc_handler::Context) {
        ctx.resources
            .rtc
            .reset_event(hal::rtc::RtcInterrupt::Compare0);
        // ctx.resources.rtc.disable_interrupt(hal::rtc::RtcInterrupt::Compare0, None);

        ctx.resources.i2c.enable();

        let mut sensor = common::lsm303agr::LSM303AGR::new(ctx.resources.i2c);
        let meas = sensor.get_measurement().unwrap();

        core::mem::drop(sensor);
        ctx.resources.i2c.disable();

        let sensor_id: u16 = 0xAB01;
        let data: &[&[u8]] = &[
            &3u16.to_le_bytes(),
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
        radio.set_enabled(false);
    }
};
