pub mod axp192;

use esp_idf_hal::prelude::Peripherals;

pub struct M5 {
  pub axp: axp192::Axp192,
}
  
impl M5 {
  pub fn new() -> anyhow::Result<Self, esp_idf_hal::i2c::I2cError> {
    let peripherals = Peripherals::take().unwrap();
    let axp192 = axp192::Axp192::new(peripherals, axp192::BeginConf::default())?;

    return Ok(M5 {
      axp: axp192,
    });
  }
}
