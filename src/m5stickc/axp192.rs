use alloc::sync::Arc;

use embedded_hal::i2c::blocking::I2c;

use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c;
use esp_idf_hal::mutex::Mutex;

use super::misc::map;

const AXP192_ADDRESS: u8 = 0x34;

pub struct BeginConf {
  disable_ldo2: bool,
  disable_ldo3: bool,
  disable_rtc: bool,
  disable_dcdc1: bool,
  disable_dcdc3: bool,
  disable_ldo0: bool,
}

impl Default for BeginConf {
  fn default() -> Self {
    BeginConf {
      disable_ldo2: false,
      disable_ldo3: false,
      disable_rtc: false,
      disable_dcdc1: false,
      disable_dcdc3: false,
      disable_ldo0: false,
    }
  }
}

pub struct Axp192 {
  wire: Arc<Mutex<i2c::Master<i2c::I2C1, Gpio21<Unknown>, Gpio22<Unknown>>>>,
}

impl Axp192 {
  pub fn new(
    wire: Arc<Mutex<i2c::Master<i2c::I2C1, Gpio21<Unknown>, Gpio22<Unknown>>>>,
    x: BeginConf,
  ) -> anyhow::Result<Self, esp_idf_hal::i2c::I2cError> {
    let mut ret = Self { wire: wire };

    // Set LDO2 & LDO3(TFT_LED & TFT) 3.0V
    ret.write(&[0x28, 0xcc])?;

    // Set ADC sample rate to 200hz
    ret.write(&[0x84, 0b11110010])?;

    // Set ADC to All Enable
    ret.write(&[0x82, 0xff])?;

    // Bat charge voltage to 4.2, Current 100MA
    ret.write(&[0x33, 0xc0])?;

    // Depending on configuration enable LDO2, LDO3, DCDC1, DCDC3.
    let mut buf: u8 = (ret.read1byte(0x12)? & 0xef) | 0x4D;
    if x.disable_ldo3 {
      buf &= !(1 << 3);
    }
    if x.disable_ldo2 {
      buf &= !(1 << 2);
    }
    if x.disable_dcdc3 {
      buf &= !(1 << 1);
    }
    if x.disable_dcdc1 {
      buf &= !(1 << 0);
    }
    ret.write(&[0x12, buf])?;

    // 128ms power on, 4s power off
    ret.write(&[0x36, 0x0C])?;

    if !x.disable_ldo0 {
      // Set MIC voltage to 2.8V
      ret.write(&[0x91, 0xA0])?;

      // Set GPIO0 to LDO
      ret.write(&[0x90, 0x02])?;
    } else {
      ret.write(&[0x90, 0x07])?; // GPIO0 floating
    }

    // Disable vbus hold limit
    ret.write(&[0x30, 0x80])?;

    // Set temperature protection
    ret.write(&[0x39, 0xfc])?;

    // Enable RTC BAT charge
    ret.write(
      &[0x35, 0xa2 & (if x.disable_rtc { 0x7F } else { 0xFF })],
    )?;

    // Enable bat detection
    ret.write(&[0x32, 0x46])?;

    // Set Power off voltage 3.0v
    let buf = ret.read1byte(0x31)?;
    ret.write(&[0x31, (buf & 0xf8) | (1 << 2)])?;

    return Ok(ret);
  }

  pub fn screen_breath(&mut self, brightness: i16) -> Result<(), esp_idf_hal::i2c::I2cError> {
    if brightness > 100 || brightness < 0 {
      // TODO: return error
      return Ok(());
    }
    let vol = map(brightness.into(), 0, 100, 2500, 3200);
    let vol = if vol < 1800 { 0 } else { (vol - 1800) / 100 };
    let vol = ((vol as u16) << 4) as u8;

    let buf = self.read1byte(0x28)?;
    self.write(&[0x28, ((buf & 0x0f) | vol)])?;

    return Ok(());
  }

  pub fn set_sleep(&mut self) -> Result<(), esp_idf_hal::i2c::I2cError> {
    let data = self.read1byte(0x31)?;
    self.write(&[0x31, data | (1 << 3)])?; // Turn on short press to wake up
    let data = self.read1byte(0x90)?;
    self.write(&[0x90 , data | 0x07])?; // GPIO0 floating
    self.write(&[0x82, 0x00])?; // Disable ADCs
    let data = self.read1byte(0x12)?;
    self.write(&[0x12, data & 0xA1])?; // Disable all outputs but DCDC1

    return Ok(());
  }

  pub fn get_bat_voltage(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 1.1 / 1000.0;
    let data = self.read12bit(0x78)? as f32;
    return Ok(data * ADCLSB);
  }
  pub fn get_bat_current(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.5;
    let current_in = self.read13bit(0x7A)? as f32;
    let current_out = self.read13bit(0x7C)? as f32;
    return Ok((current_in - current_out) * ADCLSB);
  }

  pub fn get_vbus_voltage(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 1.7 / 1000.0;
    let data = self.read12bit(0x5A)? as f32;
    return Ok(data * ADCLSB);
  }

  pub fn get_vbus_current(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.375;
    let data = self.read12bit( 0x5C )? as f32;
    return Ok(data * ADCLSB);
  }

  pub fn get_temp_in_axp192(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.1;
    const OFFSET_DEG_C: f32 = -144.7;
    let data = self.read12bit(0x5E)?;
    return Ok(OFFSET_DEG_C + (data as f32) * ADCLSB);
  }

  fn write(&mut self, bytes: &[u8]) -> anyhow::Result<(), esp_idf_hal::i2c::I2cError> {
    let mut wire = self.wire.lock();
    wire.write(AXP192_ADDRESS, bytes)?;
    return Ok(());
  }

  fn read1byte(&mut self, addr: u8) -> anyhow::Result<u8, esp_idf_hal::i2c::I2cError> {
    let mut wire = self.wire.lock();
    let mut buf = [0x00u8];
    wire.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    return Ok(buf[0]);
  }

  fn read12bit(&mut self, addr: u8) -> anyhow::Result<u16, esp_idf_hal::i2c::I2cError> {
    let mut wire = self.wire.lock();
    let mut buf = [0x00u8, 0x00u8];
    wire.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    return Ok(((buf[0] as u16) << 4) + (buf[1] as u16));
  }

  fn read13bit(&mut self, addr: u8) -> anyhow::Result<u16, esp_idf_hal::i2c::I2cError> {
    let mut wire = self.wire.lock();
    let mut buf = [0x00u8, 0x00u8];
    wire.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    return Ok(((buf[0] as u16) << 5) + (buf[1] as u16));
  }
}
