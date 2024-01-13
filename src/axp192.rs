use esp_idf_hal::i2c::I2cError;

use super::misc::map;

const AXP192_ADDRESS: u8 = 0x34;

pub struct Axp192<I2C> {
  i2c: I2C,
}

impl<I2C> Axp192<I2C>
where
  I2C: embedded_hal::i2c::I2c,
  I2cError: From<<I2C as embedded_hal::i2c::ErrorType>::Error>,
{
  pub fn new(i2c: I2C) -> anyhow::Result<Self, esp_idf_hal::i2c::I2cError> {
    let mut ret = Self { i2c };

    // Set LDO2 & LDO3(TFT_LED & TFT) 3.0V
    ret.write(&[0x28, 0xcc])?;

    // // Set ADC sample rate to 200hz
    // ret.write(&[0x84, 0b11110010])?;

    // Set ADC to All Enable
    ret.write(&[0x82, 0xff])?;

    // Bat charge voltage to 4.2, Current 100MA
    ret.write(&[0x33, 0xc0])?;

    // Enable Ext, LDO2, LDO3, DCDC1
    let buf: u8 = ret.read8bit(0x12)?;
    ret.write(&[0x12, buf | 0x4D])?;

    // 128ms power on, 4s power off
    ret.write(&[0x36, 0x0C])?;

    if cfg!(feature = "m5stickc_plus") {
      // Set RTC voltage to 3.3V
      ret.write(&[0x91, 0xF0])?;
    } else {
      // Set MIC voltage to 2.8V
      ret.write(&[0x91, 0xA0])?;
    };

    // Set GPIO0 to LDO
    ret.write(&[0x90, 0x02])?;

    // Disable vbus hold limit
    ret.write(&[0x30, 0x80])?;

    // Set temperature protection
    ret.write(&[0x39, 0xfc])?;

    // Enable RTC BAT charge
    ret.write(&[0x35, 0xa2])?;

    // Enable bat detection
    ret.write(&[0x32, 0x46])?;

    if cfg!(not(feature = "m5stickc_plus")) {
      // Set Power off voltage 3.0v
      let buf = ret.read8bit(0x31)?;
      ret.write(&[0x31, (buf & 0xf8) | (1 << 2)])?;
    }

    Ok(ret)
  }

  pub fn screen_breath(&mut self, brightness: i16) -> Result<(), ()> {
    if !(0..=100).contains(&brightness) {
      return Err(());
    }
    let vol = map(brightness.into(), 0, 100, 2500, 3200);
    let vol = if vol < 1800 { 0 } else { (vol - 1800) / 100 };
    let vol = ((vol as u16) << 4) as u8;

    let buf = self.read8bit(0x28)?;
    self.write(&[0x28, ((buf & 0x0f) | vol)])?;

    Ok(())
  }

  pub fn set_sleep(&mut self) -> Result<(), esp_idf_hal::i2c::I2cError> {
    let buf = self.read8bit(0x31)?;
    self.write(&[0x31, buf | (1 << 3)])?; // Turn on short press to wake up

    if cfg!(not(feature = "m5stickc_plus")) {
      self.write(&[0x90, 0x00])?;
      self.write(&[0x12, 0x09])?;
    } else {
      let data = self.read8bit(0x90)?;
      self.write(&[0x90, data | 0x07])?; // GPIO0 floating
      self.write(&[0x82, 0x00])?; // Disable ADCs
    }

    let buf = self.read8bit(0x12)?;
    self.write(&[0x12, buf & 0xA1])?; // Disable all outputs but DCDC1

    Ok(())
  }

  // 0 not press, 0x01 long press, 0x02 press
  pub fn get_btn_press(&mut self) -> Result<u8, esp_idf_hal::i2c::I2cError> {
    let state = self.read8bit(0x46)?;
    if state > 0 {
      let _ = self.write(&[0x46, 0x03]);
    }
    Ok(state)
  }

  pub fn get_bat_voltage(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 1.1 / 1000.0;
    let data = self.read12bit(0x78)? as f32;
    Ok(data * ADCLSB)
  }
  pub fn get_bat_current(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.5;
    let current_in = self.read13bit(0x7A)? as f32;
    let current_out = self.read13bit(0x7C)? as f32;
    Ok((current_in - current_out) * ADCLSB)
  }

  pub fn get_vbus_voltage(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 1.7 / 1000.0;
    let data = self.read12bit(0x5A)? as f32;
    Ok(data * ADCLSB)
  }

  pub fn get_vbus_current(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.375;
    let data = self.read12bit(0x5C)? as f32;
    Ok(data * ADCLSB)
  }

  pub fn get_temp_in_axp192(&mut self) -> anyhow::Result<f32, esp_idf_hal::i2c::I2cError> {
    const ADCLSB: f32 = 0.1;
    const OFFSET_DEG_C: f32 = -144.7;
    let data = self.read12bit(0x5E)?;
    Ok(OFFSET_DEG_C + (data as f32) * ADCLSB)
  }

  fn write(&mut self, bytes: &[u8]) -> anyhow::Result<(), esp_idf_hal::i2c::I2cError> {
    self.i2c.write(AXP192_ADDRESS, bytes)?;
    Ok(())
  }

  fn read8bit(&mut self, addr: u8) -> anyhow::Result<u8, esp_idf_hal::i2c::I2cError> {
    let mut buf = [0x00u8];
    self.i2c.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    Ok(buf[0])
  }

  fn read12bit(&mut self, addr: u8) -> anyhow::Result<u16, esp_idf_hal::i2c::I2cError> {
    let mut buf = [0x00u8; 2];
    self.i2c.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    Ok(((buf[0] as u16) << 4) + (buf[1] as u16))
  }

  fn read13bit(&mut self, addr: u8) -> anyhow::Result<u16, esp_idf_hal::i2c::I2cError> {
    let mut buf = [0x00u8; 2];
    self.i2c.write_read(AXP192_ADDRESS, &[addr], &mut buf)?;
    Ok(((buf[0] as u16) << 5) + (buf[1] as u16))
  }
}
