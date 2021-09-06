use nrf52810_pac as pac;

pub struct Saadc {
    saadc: pac::SAADC,
    p0: pac::P0,
    pin: usize,
}

impl Saadc {
    pub fn new(saadc: pac::SAADC, p0: pac::P0, pin: usize) -> Self {
        // select P0.03/AIN1 as the positive input
        p0.pin_cnf[pin].write(|w| w.input().connect());
        saadc.ch[0].pselp.write(|w| w.pselp().analog_input1());
        // set gain 1
        saadc.ch[0].config.write(|w| w.gain().gain1());
        // set max count 1
        saadc.result.maxcnt.write(|w| unsafe { w.maxcnt().bits(1) });
        // enable
        saadc.enable.write(|w| w.enable().enabled());

        Saadc { saadc, p0, pin }
    }

    pub fn getValue(&mut self) -> f32 {
        let adc_result = 0u16;

        // set result pointer
        self.saadc
            .result
            .ptr
            .write(|w| unsafe { w.ptr().bits((&adc_result as *const u16) as u32) });

        // start ADC
        self.saadc.tasks_start.write(|w| w.tasks_start().trigger());
        while self
            .saadc
            .events_started
            .read()
            .events_started()
            .is_not_generated()
        {}

        // trigger sample task
        self.saadc
            .tasks_sample
            .write(|w| w.tasks_sample().trigger());
        while self.saadc.events_end.read().events_end().is_not_generated() {}

        let res = adc_result as f32 * 0.6 / 1024.0 / 0.4;

        self.saadc
            .events_started
            .write(|w| w.events_started().not_generated());
        self.saadc
            .events_end
            .write(|w| w.events_end().not_generated());

        res
    }

    pub fn free(self) -> (pac::SAADC, pac::P0) {
        // disable SAADC
        self.saadc.enable.write(|w| w.enable().disabled());
        // disconnect pin
        self.p0.pin_cnf[self.pin].write(|w| w.input().disconnect());

        (self.saadc, self.p0)
    }
}
