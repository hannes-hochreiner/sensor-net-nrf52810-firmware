use nrf52810_pac as pac;
use pac::interrupt;

pub struct Rtc<'a> {
    rtc: &'a mut pac::RTC0,
}

impl<'a> Rtc<'a> {
    pub fn new(rtc: &'a mut pac::RTC0, nvic: &mut pac::NVIC) -> Self {
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::RTC0);
        // unsafe { cortex_m::peripheral::NVIC::unmask(pac::interrupt::RTC0) };

        Rtc { rtc }
    }

    pub fn set_prescaler(&mut self, prescaler: u16) {
        // TODO: check that the prescaler is <= 12 bits.
        self.rtc
            .prescaler
            .write(|w| unsafe { w.prescaler().bits(prescaler) });
    }

    pub fn set_compare(&mut self, compare: u32) {
        // TODO: check that the compare value is <= 24 bits.
        self.rtc.cc[0].write(|w| unsafe { w.compare().bits(compare) });
    }

    pub fn start(&self) {
        self.rtc.events_compare[0].write(|w| w.events_compare().not_generated());
        self.rtc.evtenset.write(|w| w.compare0().set());
        self.rtc.intenset.write(|w| w.compare0().set());
        self.rtc.tasks_clear.write(|w| w.tasks_clear().trigger());
        self.rtc.tasks_start.write(|w| w.tasks_start().trigger());
    }

    pub fn wait(&self) {
        while self.rtc.events_compare[0]
            .read()
            .events_compare()
            .is_not_generated()
        {
            cortex_m::asm::wfi();
        }

        self.rtc.evtenclr.write(|w| w.compare0().clear());
        self.rtc.tasks_stop.write(|w| w.tasks_stop().trigger());
    }
}

#[interrupt]
fn RTC0() {
    let device = unsafe { pac::Peripherals::steal() };
    device.RTC0.intenclr.write(|w| w.compare0().clear());
}
