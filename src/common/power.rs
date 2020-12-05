use nrf52810_hal::pac;

pub struct Power {
    power: pac::POWER,
}

pub enum Mode {
    ConstantLatency,
    LowPower
}

pub enum Event {
    None,
    SleepEnter,
    SleepExit,
}

impl Power {
    pub fn new(power: pac::POWER) -> Power {
        Power {power: power}
    }

    pub fn set_mode(&mut self, mode: Mode) {
        match mode {
            Mode::ConstantLatency => {
                self.power.tasks_constlat.write(|w| { w.tasks_constlat().set_bit() });
            },
            Mode::LowPower => {
                self.power.tasks_lowpwr.write(|w| { w.tasks_lowpwr().set_bit() });
            }
        }
    }

    pub fn set_interrupt(&mut self, event: Event) {
        match event {
            Event::SleepEnter => {
                self.power.intenset.write(|w| { w.sleepenter().set() });
            },
            Event::SleepExit => {
                self.power.intenset.write(|w| { w.sleepexit().set() });
            },
            Event::None => {}
        }
    }

    pub fn clear_interrupt(&mut self, event: Event) {
        match event {
            Event::SleepEnter => {
                self.power.intenclr.write(|w| { w.sleepenter().clear() });
            },
            Event::SleepExit => {
                self.power.intenclr.write(|w| { w.sleepexit().clear() });
            },
            Event::None => {}
        }
    }

    pub fn clear_events(&mut self) {
        self.power.events_sleepenter.write(|w| { w.events_sleepenter().clear_bit() } );
        self.power.events_sleepexit.write(|w| { w.events_sleepexit().clear_bit() } );
    }

    pub fn is_event_set(&self, event: Event) -> bool {
        match event {
            Event::SleepEnter => {
                self.power.events_sleepenter.read().events_sleepenter().is_generated()
            },
            Event::SleepExit => {
                self.power.events_sleepexit.read().events_sleepexit().is_generated()
            },
            _ => {
                false
            }
        }
    }

    pub fn get_event(&self) -> Event {
        if self.power.events_sleepexit.read().events_sleepexit().is_generated() {
            return Event::SleepExit;
        }
        
        if self.power.events_sleepenter.read().events_sleepenter().is_generated() {
            return Event::SleepEnter;
        }

        Event::None
    }
}
