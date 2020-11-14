use nrf52810_hal::pac;

pub struct Clock {
    clock: pac::CLOCK,
}

#[derive(PartialEq)]
pub enum HfClkSource {
    Rc,
    Xtal
}

pub struct HfClkStatus {
    pub source: HfClkSource,
    pub running: bool
}

impl Clock {
    pub fn new(clock: pac::CLOCK) -> Clock {
        Clock {clock: clock}
    }

    pub fn hf_clk_status(&self) -> HfClkStatus {
        let mut res = HfClkStatus {source: HfClkSource::Rc, running: false};
        
        if self.clock.hfclkstat.read().src().bit() {
            res.source = HfClkSource::Xtal;
        }

        if self.clock.hfclkstat.read().state().bit() {
            res.running = true;
        }

        res
    }

    pub fn start_hf_clk_xtal(&self) {
        self.clock.intenset.write(|w| { w.hfclkstarted().set_bit() });
        self.clock.tasks_hfclkstart.write(|w| { w.tasks_hfclkstart().trigger() });
    }

    pub fn clear_all_interrupts(&self) {
        self.clock.intenclr.write(|w| {
            w.hfclkstarted().set_bit()
             .lfclkstarted().set_bit()
             .done().set_bit()
             .ctto().set_bit()
        });
    }
}
