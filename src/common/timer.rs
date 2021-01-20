use nrf52810_pac as pac;
use pac::interrupt;

pub struct Timer<S: State> {
    timer: pac::TIMER0,
    marker: core::marker::PhantomData<S>,
}

pub enum Active {}
pub enum Inactive {}

pub trait State {}
impl State for Active {}
impl State for Inactive {}

impl Timer<Inactive> {
    pub fn new(timer: pac::TIMER0, nvic: &mut pac::NVIC) -> Self {
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::TIMER0);
        timer.bitmode.write(|w| w.bitmode()._32bit());

        Timer {
            timer: timer,
            marker: core::marker::PhantomData,
        }
    }

    pub fn set_timeout_mus(&mut self, timeout: u32) {
        self.timer.cc[0].write(|w| unsafe { w.cc().bits(timeout) });
    }

    pub fn start(self) -> Timer<Active> {
        self.timer.tasks_clear.write(|w| w.tasks_clear().trigger());
        self.timer.intenset.write(|w| w.compare0().set());
        self.timer.tasks_start.write(|w| w.tasks_start().trigger());

        Timer {
            timer: self.timer,
            marker: core::marker::PhantomData,
        }
    }
}

impl Timer<Active> {
    pub fn wait(self) -> Timer<Inactive> {
        while self.timer.events_compare[0]
            .read()
            .events_compare()
            .is_not_generated()
        {
            cortex_m::asm::wfi();
        }

        self.timer.tasks_stop.write(|w| w.tasks_stop().trigger());
        self.timer.events_compare[0].write(|w| w.events_compare().not_generated());

        Timer {
            timer: self.timer,
            marker: core::marker::PhantomData,
        }
    }
}

#[interrupt]
fn TIMER0() {
    let device = unsafe { pac::Peripherals::steal() };
    device.TIMER0.intenclr.write(|w| w.compare0().clear());
}
