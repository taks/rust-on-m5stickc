use embedded_hal::i2c::blocking::I2c;

use esp_idf_hal::i2c;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

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

pub fn begin(x: BeginConf) -> anyhow::Result<(), esp_idf_hal::i2c::I2cError> {
  let peripherals = Peripherals::take().unwrap();
  let i2c = peripherals.i2c1;
  let sda = peripherals.pins.gpio21;
  let scl = peripherals.pins.gpio22;

  let config = <i2c::config::MasterConfig as Default>::default().baudrate(400.kHz().into());
  let mut i2c = i2c::Master::<i2c::I2C1, _, _>::new(i2c, i2c::MasterPins { sda, scl }, config)?;

  // Set LDO2 & LDO3(TFT_LED & TFT) 3.0V
  i2c.write(AXP192_ADDRESS, &[0x28, 0xcc])?;

  // Set ADC sample rate to 200hz
  i2c.write(AXP192_ADDRESS, &[0x84, 0b11110010])?;

  // Set ADC to All Enable
  i2c.write(AXP192_ADDRESS, &[0x82, 0xff])?;

  // Bat charge voltage to 4.2, Current 100MA
  i2c.write(AXP192_ADDRESS, &[0x33, 0xc0])?;

  // Depending on configuration enable LDO2, LDO3, DCDC1, DCDC3.
  let mut buffer = [0x00];
  i2c.write_read(AXP192_ADDRESS, &[0x12], &mut buffer)?;
  let mut buf: u8 = (buffer[0] & 0xef) | 0x4D;
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
  i2c.write(AXP192_ADDRESS, &[0x12, buf])?;

  // 128ms power on, 4s power off
  i2c.write(AXP192_ADDRESS, &[0x36, 0x0C])?;

  if !x.disable_ldo0 {
    // Set MIC voltage to 2.8V
    i2c.write(AXP192_ADDRESS, &[0x91, 0xA0])?;

    // Set GPIO0 to LDO
    i2c.write(AXP192_ADDRESS, &[0x90, 0x02])?;
  } else {
    i2c.write(AXP192_ADDRESS, &[0x90, 0x07])?; // GPIO0 floating
  }

  // Disable vbus hold limit
  i2c.write(AXP192_ADDRESS, &[0x30, 0x80])?;

  // Set temperature protection
  i2c.write(AXP192_ADDRESS, &[0x39, 0xfc])?;

  // Enable RTC BAT charge
  i2c.write(
    AXP192_ADDRESS,
    &[0x35, 0xa2 & (if x.disable_rtc { 0x7F } else { 0xFF })],
  )?;

  // Enable bat detection
  i2c.write(AXP192_ADDRESS, &[0x32, 0x46])?;

  // Set Power off voltage 3.0v
  i2c.write_read(AXP192_ADDRESS, &[0x31], &mut buffer)?;

  i2c.write(AXP192_ADDRESS, &[0x31, (buffer[0] & 0xf8) | (1 << 2)])?;

  return Ok(());
}
