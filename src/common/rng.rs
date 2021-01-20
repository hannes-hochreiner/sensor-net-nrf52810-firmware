use nrf52810_pac as pac;
use pac::interrupt;

pub struct Rng<S: State> {
    rng: pac::RNG,
    marker: core::marker::PhantomData<S>,
}
pub enum Active {}
pub enum Inactive {}

pub trait State {}
impl State for Active {}
impl State for Inactive {}

impl Rng<Inactive> {
    pub fn new(rng: pac::RNG, nvic: &mut pac::NVIC) -> Self {
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::RNG);

        Rng {
            rng: rng,
            marker: core::marker::PhantomData,
        }
    }

    pub fn start_getting_value(self) -> Rng<Active> {
        self.rng.intenset.write(|w| w.valrdy().set());
        self.rng.tasks_start.write(|w| w.tasks_start().trigger());

        Rng {
            rng: self.rng,
            marker: core::marker::PhantomData,
        }
    }
}

impl Rng<Active> {
    pub fn wait_for_value(self) -> (Rng<Inactive>, u8) {
        while self
            .rng
            .events_valrdy
            .read()
            .events_valrdy()
            .is_not_generated()
        {
            cortex_m::asm::wfi();
        }

        self.rng
            .events_valrdy
            .write(|w| w.events_valrdy().not_generated());
        let res = self.rng.value.read().value().bits();

        (
            Rng {
                rng: self.rng,
                marker: core::marker::PhantomData,
            },
            res,
        )
    }
}

#[interrupt]
fn RNG() {
    let device = unsafe { pac::Peripherals::steal() };
    device.RNG.intenclr.write(|w| w.valrdy().clear());
}
