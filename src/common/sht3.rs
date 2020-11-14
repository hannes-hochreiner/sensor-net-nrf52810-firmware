use embedded_hal::blocking::{i2c as i2c, delay as delay};

#[derive(core::fmt::Debug)]
pub enum Sht3Error<E> {
    EmbeddedError(E),
    IdError,
    CrcError
}

pub struct SHT3<'a, I2C, DELAY> {
    i2c: &'a mut I2C,
    delay: &'a mut DELAY,
    address: u8
}

pub struct Measurement {
    temperature: f32,
    humidity: f32
}

const COM_WAKEUP: [u8; 2] = [0x35, 0x17];
const COM_ID : [u8; 2] = [0xEF, 0xC8];
const COM_RESET : [u8; 2] = [0x80, 0x5D];
const COM_SLEEP : [u8; 2] = [0xB0, 0x98];
const COM_MEAS_TH : [u8; 2] = [0x78, 0x66];
const COM_MEAS_TH_LP : [u8; 2] = [0x60, 0x9C];

impl<'a, I2C, DELAY, E> SHT3<'a, I2C, DELAY> where 
I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
DELAY: delay::DelayMs<u8>,
E: core::fmt::Debug {
    pub fn new(i2c: &'a mut I2C, delay: &'a mut DELAY) -> SHT3<'a, I2C, DELAY> {
        SHT3 {i2c: i2c, delay: delay, address: 0x70}
    }

    pub fn init(&mut self) -> Result<(), Sht3Error<E>> {
        self.i2c.write(self.address, &COM_WAKEUP).map_err(Sht3Error::EmbeddedError).unwrap();
        self.delay.delay_ms(1);
        self.i2c.write(self.address, &COM_ID).map_err(Sht3Error::EmbeddedError).unwrap();
        let mut buf_id : [u8; 3] = [0; 3];
        self.i2c.read(self.address, &mut buf_id).map_err(Sht3Error::EmbeddedError).unwrap();
        self.i2c.write(self.address, &COM_SLEEP).map_err(Sht3Error::EmbeddedError).unwrap();
        
        if (buf_id[0] & 0b0000_1000 != 0b0000_1000) || (buf_id[1] & 0b0011_1111 != 0b0000_0111) {
            return Err(Sht3Error::IdError);
        }

        if self.crc8(&buf_id[0..2]) != buf_id[2] {
            return Err(Sht3Error::CrcError);
        }

        Ok(())
    }

    pub fn get_measurement(&mut self) -> Result<Measurement, Sht3Error<E>> {
        self.i2c.write(self.address, &COM_WAKEUP).map_err(Sht3Error::EmbeddedError).unwrap();
        self.delay.delay_ms(1);
        self.i2c.write(self.address, &COM_MEAS_TH).map_err(Sht3Error::EmbeddedError).unwrap();
        let mut buf_meas : [u8; 6] = [0; 6];
        self.delay.delay_ms(15);
        self.i2c.read(self.address, &mut buf_meas).map_err(Sht3Error::EmbeddedError).unwrap();
        self.i2c.write(self.address, &COM_SLEEP).map_err(Sht3Error::EmbeddedError).unwrap();

        if (self.crc8(&buf_meas[0..2]) != buf_meas[2]) || (self.crc8(&buf_meas[3..5]) != buf_meas[5]) {
            return Err(Sht3Error::CrcError);
        }

        let hum_array = [buf_meas[3], buf_meas[4]];
        let hum = u16::from_be_bytes(hum_array) as f32 / 655.36;
        let temp_array = [buf_meas[0], buf_meas[1]];
        let temp = -45.0 + 0.00267028808594 * u16::from_be_bytes(temp_array) as f32;

        Ok(Measurement {temperature: temp, humidity: hum})
    }

    fn crc8(&self, buffer: &[u8]) -> u8 {
        let mut rem = 0xff;
        let poly = 0x31;

        for byte in buffer {
            rem = rem ^ byte;

            for _i in 1..8 {
                if rem & 0x80 == 0x80 {
                    rem = (rem << 1) ^ poly;
                } else {
                    rem = rem << 1;
                }
            }
        }

        rem
    }
}
