use super::timer;
use super::twim;

#[derive(core::fmt::Debug)]
pub enum Sht4xError {
    IdError,
    CrcError,
    TimerError,
}

pub struct SHT4X<'a> {
    twim: &'a mut twim::Twim<'a>,
    timer: &'a mut timer::Timer<'a>,
    address: u8,
    buffer: [u8; 1],
}

pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

impl<'a> SHT4X<'a> {
    pub fn new(
        twim: &'a mut twim::Twim<'a>,
        timer: &'a mut timer::Timer<'a>,
        address: u8,
    ) -> SHT4X<'a> {
        SHT4X {
            twim,
            timer,
            address: address,
            buffer: [0; 1],
        }
    }

    pub fn start_reading_serial(&mut self) -> Result<(), Sht4xError> {
        self.buffer[0] = 0x89;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        // TODO: check that timer is not running already
        self.timer.set_timeout_mus(1_000);
        self.timer.start();

        Ok(())
    }

    pub fn start_measurement(&mut self) -> Result<(), Sht4xError> {
        self.buffer[0] = 0xFD;
        self.twim.start_write(self.address, &self.buffer);
        self.twim.wait().unwrap();

        // TODO: check that the timer is not running already
        self.timer.set_timeout_mus(10_000);
        self.timer.start();

        Ok(())
    }

    pub fn wait_for_serial(&mut self) -> Result<u32, Sht4xError> {
        // check that timer is active
        self.timer.wait();
        let mut buffer: [u8; 6] = [0; 6];
        self.twim.start_read(self.address, &mut buffer);
        self.twim.wait().unwrap();

        if buffer[2] != crc8(&buffer[0..2]) || buffer[5] != crc8(&buffer[3..5]) {
            return Err(Sht4xError::CrcError);
        }

        Ok(u32::from_be_bytes([
            buffer[0], buffer[1], buffer[3], buffer[4],
        ]))
    }

    pub fn wait_for_measurement(&mut self) -> Result<Measurement, Sht4xError> {
        // check that timer is active
        self.timer.wait();
        let mut buffer: [u8; 6] = [0; 6];
        self.twim.start_read(self.address, &mut buffer);
        self.twim.wait().unwrap();

        if buffer[2] != crc8(&buffer[0..2]) || buffer[5] != crc8(&buffer[3..5]) {
            return Err(Sht4xError::CrcError);
        }

        let temperature =
            -45.0 + 175.0 * u16::from_be_bytes([buffer[0], buffer[1]]) as f32 / 65535.0;
        let humidity = -6.0 + 125.0 * u16::from_be_bytes([buffer[3], buffer[4]]) as f32 / 65535.0;

        Ok(Measurement {
            temperature,
            humidity,
        })
    }
}

fn crc8(buffer: &[u8]) -> u8 {
    let mut rem = 0xff;
    let poly = 0x31;

    for byte in buffer {
        rem ^= byte;

        for _i in 1..9 {
            if rem & 0x80 == 0x80 {
                rem = (rem << 1) ^ poly;
            } else {
                rem = rem << 1;
            }
        }
    }

    rem
}
