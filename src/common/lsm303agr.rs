use embedded_hal::blocking::i2c as i2c;

#[derive(core::fmt::Debug)]
pub enum Lsm303agrError<E> {
    EmbeddedError(E),
    IdError,
    MeasurementTimeoutError
}

pub struct LSM303AGR<'a, I2C> {
    i2c: &'a mut I2C,
    address_acc: u8,
    address_mag: u8
}

pub struct Measurement {
    pub acc_x: i16,
    pub acc_y: i16,
    pub acc_z: i16,
    pub mag_x: i16,
    pub mag_y: i16,
    pub mag_z: i16,
}

const WHO_AM_I_A: u8 = 0x0F;
const CTRL_REG1_A: u8 = 0x20;
const CTRL_REG4_A: u8 = 0x23;
const CTRL_REG5_A: u8 = 0x24;
const STATUS_REG_A: u8 = 0x27;
const OUT_X_L_A: u8 = 0x28;

const WHO_AM_I_M: u8 = 0x4F;
const CFG_REG_A_M: u8 = 0x60;
const STATUS_REG_M: u8 = 0x67;
const OUTX_L_REG_M: u8 = 0x68;

impl<'a, I2C, E> LSM303AGR<'a, I2C> where 
I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
E: core::fmt::Debug {
    pub fn new(i2c: &'a mut I2C) -> LSM303AGR<'a, I2C> {
        LSM303AGR {
            i2c: i2c,
            address_acc: 0b0011001,
            address_mag: 0b0011110
        }
    }

    pub fn init(&mut self) -> Result<(), Lsm303agrError<E>> {
        let mut acc_id = [0u8];

        self.i2c.write(self.address_acc, &[WHO_AM_I_A]).map_err(Lsm303agrError::EmbeddedError).unwrap();
        self.i2c.read(self.address_acc, &mut acc_id).map_err(Lsm303agrError::EmbeddedError).unwrap();

        if acc_id[0] != 0b00110011 {
            return Err(Lsm303agrError::IdError);
        }

        // reboot acc
        self.i2c.write(self.address_acc, &[CTRL_REG5_A, 0b10000000]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        let mut mag_id = [0u8];

        self.i2c.write(self.address_mag, &[WHO_AM_I_M]).map_err(Lsm303agrError::EmbeddedError).unwrap();
        self.i2c.read(self.address_mag, &mut mag_id).map_err(Lsm303agrError::EmbeddedError).unwrap();

        if mag_id[0] != 0b01000000 {
            return Err(Lsm303agrError::IdError);
        }

        // reboot and reset mag
        self.i2c.write(self.address_mag, &[CFG_REG_A_M, 0b01100011]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        Ok(())
    }

    pub fn get_measurement(&mut self) -> Result<Measurement, Lsm303agrError<E>> {
        // start mag measurement
        // CFG_REG_A_M
        // | 0b1 temperature compensation | 0b0 reboot | 0b0 reset | 0b0 low-power mode | 0b11 output data rate | 0b00 continuous mode |
        self.i2c.write(self.address_mag, &[CFG_REG_A_M, 0b10001100]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        // start acc measurement
        // CTRL_REG4_A
        // | 0b1 block update | 0b0 endian | 0b00 full scale 2g | 0b1 high-res | 0b00 self-test off | 0b0 SPI off |
        self.i2c.write(self.address_acc, &[CTRL_REG4_A, 0b10001000]).map_err(Lsm303agrError::EmbeddedError).unwrap();
        // CTRL_REG1_A
        // | 0b0101 100 Hz | 0b0 (default) low power mode off | 0b111 (default) axis Z,Y,X enabled |
        self.i2c.write(self.address_acc, &[CTRL_REG1_A, 0b01010111]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        let mut _tmp = [0];
        self.i2c.write(self.address_mag, &[CFG_REG_A_M]).map_err(Lsm303agrError::EmbeddedError).unwrap();
        self.i2c.read(self.address_mag, &mut _tmp).map_err(Lsm303agrError::EmbeddedError).unwrap();

        // poll mag measurement
        let timeout_mag_max = 100;
        let mut timeout_mag = 0;
        let mut status_mag = [0];

        while timeout_mag < timeout_mag_max && status_mag[0] & 0x08 != 0x08 {
            self.i2c.write(self.address_mag, &[STATUS_REG_M]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_mag, &mut status_mag).map_err(Lsm303agrError::EmbeddedError).unwrap();
            timeout_mag += 1;
        }

        if timeout_mag == timeout_mag_max {
            return Err(Lsm303agrError::MeasurementTimeoutError);
        }

        // poll acc measurement
        let timeout_acc_max = 30;
        let mut timeout_acc = 0;
        let mut status_acc = [0];

        while timeout_acc < timeout_acc_max && status_acc[0] & 0x08 != 0x08 {
            self.i2c.write(self.address_acc, &[STATUS_REG_A]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_acc, &mut status_acc).map_err(Lsm303agrError::EmbeddedError).unwrap();
            timeout_acc += 1;
        }

        if timeout_acc == timeout_acc_max {
            return Err(Lsm303agrError::MeasurementTimeoutError);
        }

        // shutdown mag
        // CFG_REG_A_M
        // | 0b1 temperature compensation | 0b0 reboot | 0b0 reset | 0b0 low-power mode | 0b11 output data rate | 0b11 idle |
        // enable mag low-pass filter
        self.i2c.write(self.address_mag, &[CFG_REG_A_M, 0b10001111]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        // shutdown acc
        // CTRL_REG1_A
        // | 0b0000 power down | 0b0 (default) low power mode off | 0b111 (default) axis Z,Y,X enabled |
        self.i2c.write(self.address_acc, &[CTRL_REG1_A, 0b00000111]).map_err(Lsm303agrError::EmbeddedError).unwrap();

        // read mag measurement
        let mut buffer_meas_mag = [0; 6];

        while status_mag[0] != 0 {
            self.i2c.write(self.address_mag, &[OUTX_L_REG_M + 0x80]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_mag, &mut buffer_meas_mag).map_err(Lsm303agrError::EmbeddedError).unwrap();

            self.i2c.write(self.address_mag, &[STATUS_REG_M]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_mag, &mut status_mag).map_err(Lsm303agrError::EmbeddedError).unwrap();
        }

        // read acc measurement
        let mut buffer_meas_acc = [0; 6];

        while status_acc[0] != 0 {
            self.i2c.write(self.address_acc, &[OUT_X_L_A + 0x80]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_acc, &mut buffer_meas_acc).map_err(Lsm303agrError::EmbeddedError).unwrap();
    
            self.i2c.write(self.address_acc, &[STATUS_REG_A]).map_err(Lsm303agrError::EmbeddedError).unwrap();
            self.i2c.read(self.address_acc, &mut status_acc).map_err(Lsm303agrError::EmbeddedError).unwrap();
        }

        Ok(Measurement {
            acc_x: i16::from_le_bytes(self.sign_extend([buffer_meas_acc[0], buffer_meas_acc[1]])),
            acc_y: i16::from_le_bytes(self.sign_extend([buffer_meas_acc[2], buffer_meas_acc[3]])),
            acc_z: i16::from_le_bytes(self.sign_extend([buffer_meas_acc[4], buffer_meas_acc[5]])),
            mag_x: i16::from_le_bytes([buffer_meas_mag[0], buffer_meas_mag[1]]),
            mag_y: i16::from_le_bytes([buffer_meas_mag[2], buffer_meas_mag[3]]),
            mag_z: i16::from_le_bytes([buffer_meas_mag[4], buffer_meas_mag[5]]),
        })
    }

    /// Sign extend a 12-bit left aligned little endian two's complements number to 16-bit
    fn sign_extend(&self, bytes: [u8; 2]) -> [u8; 2] {
        let mut tmp = u16::from_le_bytes(bytes) >> 4;

        if tmp & 0x0800 == 0x0800 {
            tmp = tmp | 0xF000;
        }

        tmp.to_le_bytes()
    }
}
