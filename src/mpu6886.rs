
use embedded_hal::delay::blocking::DelayUs;

use esp_idf_hal::i2c::I2cError;

const MPU6886_ADDRESS: u8 = 0x68;
const MPU6886_WHOAMI: u8 = 0x75;
const MPU6886_SMPLRT_DIV: u8 = 0x19;
const MPU6886_CONFIG: u8 = 0x1A;
const MPU6886_GYRO_CONFIG: u8 = 0x1B;
const MPU6886_ACCEL_CONFIG: u8 = 0x1C;
const MPU6886_ACCEL_CONFIG2: u8 = 0x1D;
const MPU6886_FIFO_EN: u8 = 0x23;
const MPU6886_INT_PIN_CFG: u8 = 0x37;
const MPU6886_INT_ENABLE: u8 = 0x38;
const MPU6886_USER_CTRL: u8 = 0x6A;
const MPU6886_PWR_MGMT_1: u8 = 0x6B;
const MPU6886_ACCEL_XOUT_H: u8 = 0x3B;
const MPU6886_GYRO_XOUT_H: u8 = 0x43;

#[derive(Clone, Copy, Debug)]
pub enum Ascale {
  Afs2g = 0,
  Afs4g = 1,
  Afs8g = 2,
  Afs16g = 3,
}

#[derive(Clone, Copy, Debug)]
pub enum Gscale {
  Gfs250dps = 0,
  Gfs500dps = 1,
  Gfs1000dps = 2,
  Gfs2000dps = 3,
}

pub struct MPU6886<I2C> {
  i2c: I2C,
  g_res: f32,
  a_res: f32,
}

impl<I2C> MPU6886<I2C>
where
  I2C: embedded_hal::i2c::blocking::I2c,
  I2cError: From<<I2C as embedded_hal::i2c::ErrorType>::Error>,
{
  pub fn new(i2c: I2C) -> Self {
    return Self {
      i2c,
      g_res: 0.0,
      a_res: 0.0,
    };
  }

  pub fn init(&mut self) -> Result<(), esp_idf_hal::i2c::I2cError> {
    let mut delay = esp_idf_hal::delay::Ets {};

    let mut buf = [0x00u8];
    self.i2c.write_read(MPU6886_ADDRESS, &[MPU6886_WHOAMI], &mut buf)?;

    if buf[0] != 0x19 {
      return Err(esp_idf_hal::i2c::I2cError::other(
        esp_idf_sys::EspError::from(-1).unwrap(),
      ));
    }
    
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_PWR_MGMT_1, 0x00])?;
    delay.delay_ms(10).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_PWR_MGMT_1, 0x01 << 7])?;
    delay.delay_ms(10).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_PWR_MGMT_1, 0x01 << 0])?;
    delay.delay_ms(10).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_ACCEL_CONFIG, 0x10])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_GYRO_CONFIG, 0x18])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_CONFIG, 0x01])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_SMPLRT_DIV, 0x05])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_INT_ENABLE, 0x00])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_ACCEL_CONFIG2, 0x00])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_USER_CTRL, 0x00])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_FIFO_EN, 0x00])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_INT_PIN_CFG, 0x22])?;
    delay.delay_ms(1).unwrap();

    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_INT_ENABLE, 0x01])?;

    delay.delay_ms(100).unwrap();

    self.set_gyro_fsr(Gscale::Gfs2000dps)?;
    delay.delay_ms(10).unwrap();
    self.set_accel_fsr(Ascale::Afs8g)?;

    return Ok(());
  }

  pub fn set_gyro_fsr(&mut self, scale: Gscale) -> Result<(), esp_idf_hal::i2c::I2cError> {
    let regdata = (scale as u8) << 3;
    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_GYRO_CONFIG, regdata])?;

    self.g_res = match scale {
      Gscale::Gfs250dps => 250.0 / 32768.0,
      Gscale::Gfs500dps => 500.0 / 32768.0,
      Gscale::Gfs1000dps => 1000.0 / 32768.0,
      Gscale::Gfs2000dps => 2000.0 / 32768.0,
    };

    return Ok(());
  }

  pub fn set_accel_fsr(&mut self, scale: Ascale) -> Result<(), esp_idf_hal::i2c::I2cError> {
    let regdata = (scale as u8) << 3;
    self.i2c.write(MPU6886_ADDRESS, &[MPU6886_ACCEL_CONFIG, regdata])?;

    self.a_res = match scale {
      Ascale::Afs2g => 2.0 / 32768.0,
      Ascale::Afs4g => 4.0 / 32768.0,
      Ascale::Afs8g => 8.0 / 32768.0,
      Ascale::Afs16g => 16.0 / 32768.0,
    };

    return Ok(());
  }

  pub fn get_gyro_data(&mut self) -> Result<(f32, f32, f32), esp_idf_hal::i2c::I2cError> {
    let mut buf = [0x00u8; 6];

    self.i2c.write_read(MPU6886_ADDRESS, &[MPU6886_GYRO_XOUT_H], &mut buf)?;

    let gx = (((buf[0] as u16) << 8) | (buf[1] as u16)) as i16;
    let gy = (((buf[2] as u16) << 8) | (buf[3] as u16)) as i16;
    let gz = (((buf[4] as u16) << 8) | (buf[5] as u16)) as i16;

    return Ok((
      (gx as f32) * self.g_res,
      (gy as f32) * self.g_res,
      (gz as f32) * self.g_res,
    ));
  }

  pub fn get_accel_data(&mut self) -> Result<(f32, f32, f32), esp_idf_hal::i2c::I2cError> {
    let mut buf = [0x00u8; 6];

    self.i2c.write_read(MPU6886_ADDRESS, &[MPU6886_ACCEL_XOUT_H], &mut buf)?;

    let gx = (((buf[0] as u16) << 8) | (buf[1] as u16)) as i16;
    let gy = (((buf[2] as u16) << 8) | (buf[3] as u16)) as i16;
    let gz = (((buf[4] as u16) << 8) | (buf[5] as u16)) as i16;

    return Ok((
      (gx as f32) * self.a_res,
      (gy as f32) * self.a_res,
      (gz as f32) * self.a_res,
    ));
  }
}
