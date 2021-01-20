use nrf52810_pac as pac;

pub struct P0 {
    p0: pac::P0,
}

pub enum Dir {
    Input,
    Output,
}

pub enum Pull {
    Disabled,
    PullUp,
    PullDown,
}

pub enum Drive {
    S0S1,
    H0S1,
    S0H1,
    H0H1,
    D0S1,
    D0H1,
    S0D1,
    H0D1,
}

pub enum Sense {
    Disabled,
    High,
    Low,
}

pub enum Input {
    Connect,
    Disconnect,
}

impl P0 {
    pub fn new(p0: pac::P0) -> P0 {
        P0 { p0: p0 }
    }

    pub fn configure_pin(
        &mut self,
        pin: usize,
        dir: Dir,
        pull: Pull,
        drive: Drive,
        input: Input,
        sense: Sense,
    ) {
        self.p0.pin_cnf[pin].write(|mut w| {
            w = match dir {
                Dir::Output => w.dir().output(),
                Dir::Input => w.dir().input(),
            };

            w = match pull {
                Pull::Disabled => w.pull().disabled(),
                Pull::PullDown => w.pull().pulldown(),
                Pull::PullUp => w.pull().pullup(),
            };

            w = match drive {
                Drive::D0H1 => w.drive().d0h1(),
                Drive::D0S1 => w.drive().d0s1(),
                Drive::H0D1 => w.drive().h0d1(),
                Drive::H0H1 => w.drive().h0h1(),
                Drive::H0S1 => w.drive().h0s1(),
                Drive::S0D1 => w.drive().s0d1(),
                Drive::S0H1 => w.drive().s0h1(),
                Drive::S0S1 => w.drive().s0s1(),
            };

            w = match input {
                Input::Connect => w.input().connect(),
                Input::Disconnect => w.input().disconnect(),
            };

            w = match sense {
                Sense::Disabled => w.sense().disabled(),
                Sense::High => w.sense().high(),
                Sense::Low => w.sense().low(),
            };

            w
        });
    }
}
