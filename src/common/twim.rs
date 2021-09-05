use nrf52810_pac as pac;
use pac::interrupt;

pub struct Twim<'a> {
    twim: &'a mut pac::TWIM0,
    p0: &'a mut pac::P0,
    scl: usize,
    sda: usize,
}

pub enum Frequency {
    K100,
    K250,
    K400,
}

#[derive(Debug)]
pub enum Error {
    Transmit,
}

impl<'a> Twim<'a> {
    pub fn new(
        twim: &'a mut pac::TWIM0,
        nvic: &mut pac::NVIC,
        p0: &'a mut pac::P0,
        scl: usize,
        sda: usize,
        freq: Frequency,
    ) -> Twim<'a> {
        // configure pins
        p0.pin_cnf[scl].write(|mut w| {
            w.dir()
                .input()
                .pull()
                .pullup()
                .drive()
                .s0d1()
                .input()
                .connect()
                .sense()
                .disabled()
        });
        p0.pin_cnf[sda].write(|mut w| {
            w.dir()
                .input()
                .pull()
                .pullup()
                .drive()
                .s0d1()
                .input()
                .connect()
                .sense()
                .disabled()
        });
        #[allow(deprecated)]
        nvic.enable(pac::interrupt::TWIM0_TWIS0_TWI0);
        twim.psel
            .scl
            .write(|w| unsafe { w.pin().bits(scl as u8).connect().connected() });
        twim.psel
            .sda
            .write(|w| unsafe { w.pin().bits(sda as u8).connect().connected() });

        match freq {
            Frequency::K100 => twim.frequency.write(|w| w.frequency().k100()),
            Frequency::K250 => twim.frequency.write(|w| w.frequency().k250()),
            Frequency::K400 => twim.frequency.write(|w| w.frequency().k400()),
        }

        twim.shorts
            .write(|w| w.lastrx_stop().enabled().lasttx_stop().enabled());

        Twim { twim, p0, scl, sda }
    }

    /// Start writing to the TWI interface.
    ///
    /// This is function is not safe in the sense that the buffer could be changed or dropped
    /// before `wait` has finished.
    pub fn start_write(&mut self, address: u8, buffer: &[u8]) {
        // set address
        self.twim
            .address
            .write(|w| unsafe { w.address().bits(address) });

        // set buffer
        self.twim
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buffer.as_ptr() as u32) });
        // .write(|w| unsafe { w.ptr().bits((&self.buffer as *const u8) as u32) });
        self.twim
            .txd
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(buffer.len() as u16) });

        // set interrupts
        self.twim
            .intenset
            .write(|w| w.stopped().set().error().set());

        // enable
        self.twim.enable.write(|w| w.enable().enabled());

        // trigger write
        self.twim
            .events_error
            .write(|w| w.events_error().not_generated());
        self.twim
            .events_stopped
            .write(|w| w.events_stopped().not_generated());
        self.twim
            .tasks_starttx
            .write(|w| w.tasks_starttx().trigger());
    }

    /// Start reading to the TWI interface.
    ///
    /// This is function is not safe in the sense that the buffer could be changed or dropped
    /// before `wait` has finished.
    pub fn start_read(&mut self, address: u8, buffer: &mut [u8]) {
        // set address
        self.twim
            .address
            .write(|w| unsafe { w.address().bits(address) });

        // set buffer
        self.twim
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buffer.as_ptr() as u32) });
        self.twim
            .rxd
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(buffer.len() as u16) });

        // set interrupts
        self.twim
            .intenset
            .write(|w| w.stopped().set().error().set());

        // enable
        self.twim.enable.write(|w| w.enable().enabled());

        // trigger read
        self.twim
            .events_error
            .write(|w| w.events_error().not_generated());
        self.twim
            .events_stopped
            .write(|w| w.events_stopped().not_generated());
        self.twim
            .tasks_startrx
            .write(|w| w.tasks_startrx().trigger());
    }

    pub fn wait(&mut self) -> Result<(), Error> {
        while self
            .twim
            .events_stopped
            .read()
            .events_stopped()
            .is_not_generated()
            && self
                .twim
                .events_error
                .read()
                .events_error()
                .is_not_generated()
        {
            cortex_m::asm::wfi();
        }

        let flag_error = self.twim.events_error.read().events_error().is_generated();

        self.twim
            .events_error
            .write(|w| w.events_error().not_generated());
        self.twim
            .events_stopped
            .write(|w| w.events_stopped().not_generated());

        /////
        let tmp = self.twim.errorsrc.read().bits();
        /////

        self.twim.enable.write(|w| w.enable().disabled());

        match flag_error {
            false => Ok(()),
            true => Err(Error::Transmit),
        }
    }
}

// impl<'a> Drop for Twim<'a> {
//     fn drop(&mut self) {
//         // reset pins
//         self.p0.pin_cnf[self.scl].write(|mut w| {
//             w.dir().input()
//             .pull().disabled()
//             .drive().s0s1()
//             .input().disconnect()
//             .sense().disabled()
//         });
//         self.p0.pin_cnf[self.sda].write(|mut w| {
//             w.dir().input()
//             .pull().disabled()
//             .drive().s0s1()
//             .input().disconnect()
//             .sense().disabled()
//         });
//     }
// }

#[interrupt]
fn TWIM0_TWIS0_TWI0() {
    let device = unsafe { pac::Peripherals::steal() };
    device
        .TWIM0
        .intenclr
        .write(|w| w.stopped().clear().error().clear());
}
