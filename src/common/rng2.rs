use nrf52810_pac as pac;
use core::future::{Future};
use core::task::{Context, Poll};

pub struct Rng2 {
    rng: pac::RNG,
}

impl Rng2 {
    pub fn new(rng: pac::RNG, nvic: &mut pac::NVIC) -> Self {
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::RNG);

        Rng2 {
            rng: rng,
        }
    }
}

impl Future for Rng2 {
    type Output = u8;
    
    fn poll(self: core::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        if self.rng.events_valrdy.read().events_valrdy().is_generated() {
            Poll::Ready(self.rng.value.read().value().bits())
        } else {
            Poll::Pending
        }
    }
}
