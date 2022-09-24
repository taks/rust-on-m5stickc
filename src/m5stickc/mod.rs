pub mod axp192;
pub mod button;
pub mod misc;
pub mod mpu6886;

use alloc::sync::Arc;

use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c;
use esp_idf_hal::mutex::Mutex;
use esp_idf_hal::prelude::*;

use esp_idf_hal::prelude::Peripherals;

type Wire1 = i2c::Master<i2c::I2C1, Gpio21<Unknown>, Gpio22<Unknown>>;

pub struct M5 {
  pub axp: axp192::Axp192<Wire1>,
  pub btn_a: button::Button<Gpio37<Input>>,
  pub btn_b: button::Button<Gpio39<Input>>,
  pub mpu6886: mpu6886::MPU6886<Wire1>,
}

impl M5 {
  pub fn new() -> anyhow::Result<Self, esp_idf_hal::i2c::I2cError> {
    let peripherals = Peripherals::take().unwrap();

    let config = <i2c::config::MasterConfig as Default>::default().baudrate(400.kHz().into());
    let wire1_ = i2c::Master::<i2c::I2C1, _, _>::new(
      peripherals.i2c1,
      i2c::MasterPins {
        sda: peripherals.pins.gpio21,
        scl: peripherals.pins.gpio22,
      },
      config,
    )?;
    let wire1 = Arc::new(Mutex::new(wire1_));

    let axp = axp192::Axp192::new(wire1.clone(), axp192::BeginConf::default())?;

    let pin_a = peripherals.pins.gpio37.into_input().unwrap();
    let btn_a = button::Button::new(pin_a, true, 10);
    let pin_b = peripherals.pins.gpio39.into_input().unwrap();
    let btn_b = button::Button::new(pin_b, true, 10);

    let mpu6886 = mpu6886::MPU6886::new(wire1);

    return Ok(M5 {
      axp,
      btn_a,
      btn_b,
      mpu6886,
    });
  }

  pub fn update(&mut self) {
    self.btn_a.read();
    self.btn_b.read();
  }
}
