use nrf52810_pac as pac;
use pac::interrupt;

pub struct Timer<'a> {
    timer: &'a mut pac::TIMER0,
}

impl<'a> Timer<'a> {
    pub fn new(timer: &'a mut pac::TIMER0, nvic: &mut pac::NVIC) -> Self {
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::TIMER0);
        timer.bitmode.write(|w| w.bitmode()._32bit());

        Timer { timer }
    }

    pub fn set_timeout_mus(&mut self, timeout: u32) {
        self.timer.cc[0].write(|w| unsafe { w.cc().bits(timeout) });
    }

    pub fn start(&mut self) {
        self.timer.tasks_clear.write(|w| w.tasks_clear().trigger());
        self.timer.intenset.write(|w| w.compare0().set());
        self.timer.tasks_start.write(|w| w.tasks_start().trigger());
    }

    pub fn wait(&mut self) {
        while self.timer.events_compare[0]
            .read()
            .events_compare()
            .is_not_generated()
        {
            cortex_m::asm::wfi();
        }

        self.timer.tasks_stop.write(|w| w.tasks_stop().trigger());
        self.timer.events_compare[0].write(|w| w.events_compare().not_generated());
    }
}

#[interrupt]
fn TIMER0() {
    let device = unsafe { pac::Peripherals::steal() };
    device.TIMER0.intenclr.write(|w| w.compare0().clear());
}
