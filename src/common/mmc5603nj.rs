use super::twim;

pub struct ActiveMeasurement;
pub struct Inactive;

#[derive(core::fmt::Debug)]
pub enum Mmc5603njError {
    CrcError,
    TimerError,
}

pub struct MMC5603NJ<S> {
    twim: twim::Twim<twim::Inactive>,
    address: u8,
    buffer: [u8; 1],
    state: core::marker::PhantomData<S>,
}

impl MMC5603NJ<Inactive> {
    pub fn new(twim: twim::Twim<twim::Inactive>, address: u8) -> MMC5603NJ<Inactive> {
        MMC5603NJ {
            twim: twim,
            address: address,
            buffer: [0; 1],
            state: core::marker::PhantomData,
        }
    }

    pub fn start_measurement(mut self) -> Result<MMC5603NJ<ActiveMeasurement>, Mmc5603njError> {
        self.buffer[0] = 0x09;
        let twim = self
            .twim
            .start_write(self.address, &self.buffer)
            .wait()
            .unwrap();

        Ok(MMC5603NJ {
            twim: twim,
            address: self.address,
            buffer: self.buffer,
            state: core::marker::PhantomData,
        })
    }
}

impl MMC5603NJ<ActiveMeasurement> {
    pub fn wait_for_measurement(
        self,
        temperature: &mut f32,
    ) -> Result<MMC5603NJ<Inactive>, Mmc5603njError> {
        let mut buffer: [u8; 1] = [0; 1];
        let twim = self
            .twim
            .start_read(self.address, &mut buffer)
            .wait()
            .unwrap();

        *temperature = -75f32 * buffer[0] as f32 * 0.8f32;

        Ok(MMC5603NJ {
            twim: twim,
            address: self.address,
            buffer: self.buffer,
            state: core::marker::PhantomData,
        })
    }
}
