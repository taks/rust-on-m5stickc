pub mod axp192;
pub mod button;
pub mod misc;
pub mod mpu6886;
pub mod display;
pub mod display_buffer;

use alloc::sync::Arc;

use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c;
use esp_idf_hal::mutex::Mutex;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use esp_idf_hal::prelude::Peripherals;

type Wire1 = i2c::Master<i2c::I2C1, Gpio21<Unknown>, Gpio22<Unknown>>;
type Spi3Master = esp_idf_hal::spi::Master<
  esp_idf_hal::spi::SPI3,
  Gpio13<Unknown>,
  Gpio15<Unknown>,
  Gpio14<Unknown>,
  Gpio5<Unknown>,
>;

use anyhow::Result;

pub struct M5 {
  pub axp: axp192::Axp192<Wire1>,
  pub btn_a: button::Button<Gpio37<Input>>,
  pub btn_b: button::Button<Gpio39<Input>>,
  pub mpu6886: mpu6886::MPU6886<Wire1>,
  pub lcd: display::Display<Spi3Master, Gpio23<Output>, Gpio18<Output>>,
}

impl M5 {
  pub fn new() -> Result<Self, esp_idf_hal::i2c::I2cError> {
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

    let spi = peripherals.spi3;
    let tft_mosi = peripherals.pins.gpio15;
    // let tft_miso = peripherals.pins.gpio14; // TODO: unused?
    let tft_sclk = peripherals.pins.gpio13;
    let tft_dc = peripherals.pins.gpio23.into_output().unwrap();
    let tft_cs = peripherals.pins.gpio5;
    let tft_rst = peripherals.pins.gpio18.into_output().unwrap();
    let config = spi::config::Config::default().baudrate(27.MHz().into());
    let spi = spi::Master::<spi::SPI3, _, _, Gpio14<Unknown>, _>::new(
      spi,
      spi::Pins {
        sclk: tft_sclk,
        sdo: tft_mosi,
        sdi: None,
        cs: Some(tft_cs),
      },
      config,
    )?;
    let display = display::Display::new(spi, tft_dc, tft_rst).unwrap();

    return Ok(M5 {
      axp,
      btn_a,
      btn_b,
      mpu6886,
      lcd: display,
    });
  }

  pub fn update(&mut self) {
    self.btn_a.read();
    self.btn_b.read();
  }
}
