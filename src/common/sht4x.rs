use super::timer;
use super::twim;

#[derive(core::fmt::Debug)]
pub enum Sht4xError {
    IdError,
    CrcError,
    TimerError,
}

pub struct ActiveSerial;
pub struct ActiveMeasurement;
pub struct Inactive;

enum Timer {
    TimerActive(timer::Timer<timer::Active>),
    TimerInactive(timer::Timer<timer::Inactive>),
}

pub struct SHT4X<S> {
    twim: twim::Twim<twim::Inactive>,
    timer: Timer,
    address: u8,
    buffer: [u8; 1],
    state: core::marker::PhantomData<S>,
}

pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

impl SHT4X<Inactive> {
    pub fn new(
        twim: twim::Twim<twim::Inactive>,
        timer: timer::Timer<timer::Inactive>,
        address: u8,
    ) -> SHT4X<Inactive> {
        SHT4X {
            twim: twim,
            timer: Timer::TimerInactive(timer),
            address: address,
            buffer: [0; 1],
            state: core::marker::PhantomData,
        }
    }

    pub fn start_reading_serial(mut self) -> Result<SHT4X<ActiveSerial>, Sht4xError> {
        self.buffer[0] = 0x89;
        let twim = self
            .twim
            .start_write(self.address, &self.buffer)
            .wait()
            .unwrap();

        match self.timer {
            Timer::TimerInactive(mut timer) => {
                timer.set_timeout_mus(1_000);
                Ok(SHT4X {
                    twim: twim,
                    timer: Timer::TimerActive(timer.start()),
                    address: self.address,
                    buffer: self.buffer,
                    state: core::marker::PhantomData,
                })
            }
            Timer::TimerActive(_) => Err(Sht4xError::TimerError),
        }
    }

    pub fn start_measurement(mut self) -> Result<SHT4X<ActiveMeasurement>, Sht4xError> {
        self.buffer[0] = 0xFD;
        let twim = self
            .twim
            .start_write(self.address, &self.buffer)
            .wait()
            .unwrap();

        match self.timer {
            Timer::TimerInactive(mut timer) => {
                timer.set_timeout_mus(10_000);
                Ok(SHT4X {
                    twim: twim,
                    timer: Timer::TimerActive(timer.start()),
                    address: self.address,
                    buffer: self.buffer,
                    state: core::marker::PhantomData,
                })
            }
            Timer::TimerActive(_) => Err(Sht4xError::TimerError),
        }
    }
}

impl SHT4X<ActiveSerial> {
    pub fn wait_for_serial(self, serial: &mut u32) -> Result<SHT4X<Inactive>, Sht4xError> {
        match self.timer {
            Timer::TimerActive(timer) => {
                let timer = timer.wait();
                let mut buffer: [u8; 6] = [0; 6];
                let twim = self
                    .twim
                    .start_read(self.address, &mut buffer)
                    .wait()
                    .unwrap();

                if buffer[2] != crc8(&buffer[0..2]) || buffer[5] != crc8(&buffer[3..5]) {
                    return Err(Sht4xError::CrcError);
                }

                *serial = u32::from_be_bytes([buffer[0], buffer[1], buffer[3], buffer[4]]);

                Ok(SHT4X {
                    twim: twim,
                    timer: Timer::TimerInactive(timer),
                    address: self.address,
                    buffer: self.buffer,
                    state: core::marker::PhantomData,
                })
            }
            _ => Err(Sht4xError::TimerError),
        }
    }
}

impl SHT4X<ActiveMeasurement> {
    pub fn wait_for_measurement(
        self,
        temperature: &mut f32,
        humidity: &mut f32,
    ) -> Result<SHT4X<Inactive>, Sht4xError> {
        match self.timer {
            Timer::TimerActive(timer) => {
                let timer = timer.wait();
                let mut buffer: [u8; 6] = [0; 6];
                let twim = self
                    .twim
                    .start_read(self.address, &mut buffer)
                    .wait()
                    .unwrap();

                if buffer[2] != crc8(&buffer[0..2]) || buffer[5] != crc8(&buffer[3..5]) {
                    return Err(Sht4xError::CrcError);
                }

                *temperature =
                    -45.0 + 175.0 * u16::from_be_bytes([buffer[0], buffer[1]]) as f32 / 65535.0;
                *humidity =
                    -6.0 + 125.0 * u16::from_be_bytes([buffer[3], buffer[4]]) as f32 / 65535.0;

                Ok(SHT4X {
                    twim: twim,
                    timer: Timer::TimerInactive(timer),
                    address: self.address,
                    buffer: self.buffer,
                    state: core::marker::PhantomData,
                })
            }
            _ => Err(Sht4xError::TimerError),
        }
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
