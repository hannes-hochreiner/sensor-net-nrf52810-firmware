use nrf52810_pac as pac;

pub struct LfActive;
pub struct LfInactive;
pub struct HfActive;
pub struct HfInactive;

pub enum Source {
    RC,
    Xtal,
    Synth,
}

pub struct Clock<L, H> {
    clock: pac::CLOCK,
    lf: L,
    hf: H,
}

impl Clock<LfInactive, HfInactive> {
    pub fn new(clock: pac::CLOCK) -> Clock<LfInactive, HfInactive> {
        Clock {
            clock: clock,
            lf: LfInactive,
            hf: HfInactive,
        }
    }
}

impl<HFS> Clock<LfInactive, HFS> {
    pub fn start_lfclk(self, source: Source, bypass: bool, external: bool) -> Clock<LfActive, HFS> {
        self.clock.lfclksrc.write(|w| {
            let w = match source {
                Source::RC => w.src().rc(),
                Source::Synth => w.src().synth(),
                Source::Xtal => w.src().xtal(),
            };

            let w = match bypass {
                true => w.bypass().enabled(),
                false => w.bypass().disabled(),
            };

            let w = match external {
                true => w.external().enabled(),
                false => w.external().disabled(),
            };

            w
        });
        self.clock
            .tasks_lfclkstart
            .write(|w| w.tasks_lfclkstart().trigger());

        while self
            .clock
            .events_lfclkstarted
            .read()
            .events_lfclkstarted()
            .is_not_generated()
        {}

        Clock {
            clock: self.clock,
            lf: LfActive,
            hf: self.hf,
        }
    }
}

impl<HFS> Clock<LfActive, HFS> {
    pub fn stop_lfclk(self) -> Clock<LfInactive, HFS> {
        self.clock
            .tasks_lfclkstop
            .write(|w| w.tasks_lfclkstop().trigger());

        Clock {
            clock: self.clock,
            lf: LfInactive,
            hf: self.hf,
        }
    }
}

impl<LFS> Clock<LFS, HfInactive> {
    pub fn start_hfclk(self) -> Clock<LFS, HfActive> {
        self.clock
            .tasks_hfclkstart
            .write(|w| w.tasks_hfclkstart().trigger());

        while self
            .clock
            .events_hfclkstarted
            .read()
            .events_hfclkstarted()
            .is_not_generated()
        {}

        Clock {
            clock: self.clock,
            lf: self.lf,
            hf: HfActive,
        }
    }
}

impl<LFS> Clock<LFS, HfActive> {
    pub fn stop_hfclk(self) -> Clock<LFS, HfInactive> {
        self.clock
            .tasks_hfclkstop
            .write(|w| w.tasks_hfclkstop().trigger());

        Clock {
            clock: self.clock,
            lf: self.lf,
            hf: HfInactive,
        }
    }
}
