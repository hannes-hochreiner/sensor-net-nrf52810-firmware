use super::twim;

#[derive(core::fmt::Debug)]
pub enum Mmc5603njError {
    CrcError,
    TimerError,
    NotDoneError,
}

pub enum Mmc5603njBias {
    Set,
    Reset,
    None,
}

pub struct MMC5603NJ<'a> {
    twim: &'a mut twim::Twim<'a>,
    address: u8,
    buffer: [u8; 1],
}

impl<'a> MMC5603NJ<'a> {
    pub fn new(twim: &'a mut twim::Twim<'a>, address: u8) -> MMC5603NJ<'a> {
        MMC5603NJ {
            twim: twim,
            address: address,
            buffer: [0; 1],
        }
    }

    pub fn start_magnetic__measruement(
        &mut self,
        bias: Mmc5603njBias,
    ) -> Result<(), Mmc5603njError> {
        match bias {
            Mmc5603njBias::Set => {
                let buffer = [0x1B, 0x08];
                self.twim.start_write(self.address, &buffer);
                self.twim.wait().unwrap();
            }
            Mmc5603njBias::Reset => {
                let buffer = [0x1B, 0x10];
                self.twim.start_write(self.address, &buffer);
                self.twim.wait().unwrap();
            }
            _ => {}
        }

        let buffer = [0x1B, 0x01];
        self.twim.start_write(self.address, &buffer);
        self.twim.wait().unwrap();

        Ok(())
    }

    pub fn wait_for_magnetic_measurement(&mut self) -> Result<(f32, f32, f32), Mmc5603njError> {
        self.buffer[0] = 0x18;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        let mut stat_buffer: [u8; 1] = [0; 1];
        self.twim.start_read(self.address, &mut stat_buffer);
        self.twim.wait().unwrap();

        while stat_buffer[0] & 0x40 != 0x40 {
            self.twim.start_write(self.address, &self.buffer);
            self.twim.wait().unwrap();
            self.twim.start_read(self.address, &mut stat_buffer);
            self.twim.wait().unwrap();
        }

        self.buffer[0] = 0x00;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        let mut buffer: [u8; 6] = [0; 6];
        self.twim.start_read(self.address, &mut buffer);
        self.twim.wait().unwrap();

        let x = u16::from_be_bytes([buffer[0], buffer[1]]) as f32 / 1024f32;
        let y = u16::from_be_bytes([buffer[2], buffer[3]]) as f32 / 1024f32;
        let z = u16::from_be_bytes([buffer[4], buffer[5]]) as f32 / 1024f32;

        Ok((x, y, z))
    }

    pub fn start_temperature_measurement(&mut self) -> Result<(), Mmc5603njError> {
        let buffer = [0x1B, 0x02];
        self.twim.start_write(self.address, &buffer);
        self.twim.wait().unwrap();

        Ok(())
    }

    pub fn wait_for_temperature_measurement(&mut self) -> Result<f32, Mmc5603njError> {
        self.buffer[0] = 0x18;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        let mut stat_buffer: [u8; 1] = [0; 1];
        self.twim.start_read(self.address, &mut stat_buffer);
        self.twim.wait().unwrap();

        while stat_buffer[0] & 0x80 != 0x80 {
            self.twim.start_write(self.address, &self.buffer);
            self.twim.wait().unwrap();
            self.twim.start_read(self.address, &mut stat_buffer);
            self.twim.wait().unwrap();
        }

        self.buffer[0] = 0x09;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        let mut buffer: [u8; 1] = [0; 1];
        self.twim.start_read(self.address, &mut buffer);
        self.twim.wait().unwrap();

        let temperature = -75f32 + (buffer[0] as f32 * 0.8f32);

        Ok(temperature)
    }
}
