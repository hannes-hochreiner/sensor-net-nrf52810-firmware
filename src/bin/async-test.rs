#![no_std]
#![no_main]

use panic_halt as _;
use nrf52810_pac as pac;
use pac::{NVIC, interrupt};
use common::rng2;
use core::{cell::Cell, future, task::Waker};
use cortex_m::interrupt::Mutex;
use core::{pin::Pin, future::Future, ptr::null, task::{Context, RawWaker, RawWakerVTable}};

static THREAD_STORE: Mutex<Cell<u32>> = Mutex::new(Cell::new(0u32));
static RWV: RawWakerVTable = RawWakerVTable::new(
    clone,
    wake,
    wake_by_ref,
    drop
);

#[cortex_m_rt::entry]
fn main() -> ! {
    let device = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut rng = rng2::Rng2::new(device.RNG, &mut core.NVIC);
    // let rng_future: Pin<&mut Future<Output = u8>> = Pin::new(&mut rng);
    let mut rng_future: Pin<&mut Future<Output = u8>> = unsafe { Pin::new_unchecked(&mut rng) };
 
    loop {
        // core::future::Future.poll_fn()
        // rng_future.poll();
        let w = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &RWV)) };
        let mut ctx = Context::from_waker(&w);
        rng_future.as_mut().poll(&mut ctx);
    }
}

unsafe fn clone(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &RawWakerVTable::new(
            clone,
            wake,
            wake_by_ref,
            drop,
        )
    )
}

unsafe fn wake(_: *const ()) {}

unsafe fn wake_by_ref(_: *const ()) {}

unsafe fn drop (_: *const ()) {}
