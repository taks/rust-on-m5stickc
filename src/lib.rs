#![no_std]
#![allow(clippy::result_unit_err)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "m5stickc_plus"))]
pub mod display_st7735;
#[cfg(feature = "m5stickc_plus")]
pub mod display_st7789;

pub mod axp192;
pub mod button;
pub mod display_buffer;
pub mod misc;
pub mod mpu6886;
pub mod mutex;

use core::cell::RefCell;

use alloc::boxed::Box;
use critical_section::Mutex;
use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use anyhow::Result;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_sys::EspError;

use embedded_hal_bus::i2c;

#[cfg(not(feature = "m5stickc_plus"))]
use crate::display_st7735::Display;
#[cfg(not(feature = "m5stickc_plus"))]
type Lcd<'a> = Display<
  SpiDeviceDriver<'a, SpiDriver<'a>>,
  PinDriver<'a, Gpio23, Output>,
  PinDriver<'a, Gpio18, Output>,
>;
#[cfg(not(feature = "m5stickc_plus"))]
const SPI_BAUDRATE: u32 = 27;

#[cfg(feature = "m5stickc_plus")]
use crate::display_st7789::Display;
#[cfg(feature = "m5stickc_plus")]
type Lcd<'a> = Display<
  SpiDeviceDriver<'a, SpiDriver<'a>>,
  PinDriver<'a, Gpio23, Output>,
  PinDriver<'a, Gpio18, Output>,
  PinDriver<'a, Gpio0, Output>,
>;
#[cfg(feature = "m5stickc_plus")]
const SPI_BAUDRATE: u32 = 40;

#[inline]
#[allow(unused)]
pub(crate) unsafe fn extend_lifetime<'a, 'b: 'a, T: ?Sized>(r: &'a T) -> &'b T {
  core::mem::transmute::<&'a T, &'b T>(r)
}

pub struct M5<'a> {
  i2c1: Box<Mutex<RefCell<I2cDriver<'a>>>>,
  pub axp: axp192::Axp192<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>>,
  pub imu: mpu6886::MPU6886<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>>,
  pub btn_a: button::Button<PinDriver<'a, Gpio37, Input>>,
  pub btn_b: button::Button<PinDriver<'a, Gpio39, Input>>,
  pub lcd: Lcd<'a>,
  pub led: PinDriver<'a, Gpio10, Output>,
}

impl<'a> M5<'a> {
  pub fn new() -> Result<Self, EspError> {
    let peripherals = Peripherals::take().unwrap();

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c1 = I2cDriver::new(
      peripherals.i2c1,
      peripherals.pins.gpio21,
      peripherals.pins.gpio22,
      &config,
    )?;
    let i2c1 = Box::new(Mutex::new(RefCell::new(i2c1)));

    let i2c1_ref = unsafe { extend_lifetime(i2c1.as_ref()) };

    let axp = axp192::Axp192::new(i2c::CriticalSectionDevice::new(i2c1_ref)).unwrap();
    let mpu6886 = mpu6886::MPU6886::new(i2c::CriticalSectionDevice::new(i2c1_ref));

    let pin_a = PinDriver::input(peripherals.pins.gpio37)?;
    let btn_a = button::Button::new(pin_a, true, 10);
    let pin_b = PinDriver::input(peripherals.pins.gpio39)?;
    let btn_b = button::Button::new(pin_b, true, 10);

    let spi = peripherals.spi3;
    let tft_mosi = peripherals.pins.gpio15;
    let tft_sclk = peripherals.pins.gpio13;
    let tft_dc = PinDriver::output(peripherals.pins.gpio23)?;
    let tft_cs = peripherals.pins.gpio5;
    let tft_rst = PinDriver::output(peripherals.pins.gpio18)?;
    // let tft_driver = SpiDriver::new(peripherals.spi3, tft_sclk, tft_mosi, None, Dma::Disabled)?;

    let config = spi::config::Config::default()
      .baudrate(SPI_BAUDRATE.MHz().into())
      .write_only(true);
    let spi = SpiDeviceDriver::new_single(
      spi,
      tft_sclk,
      tft_mosi,
      None::<Gpio0>,
      Some(tft_cs),
      &SpiDriverConfig::new(),
      &config,
    )?;
    let display = Display::new(spi, tft_dc, tft_rst).unwrap();

    let mut led = PinDriver::output(peripherals.pins.gpio10)?;
    let _ = led.set_high();

    Ok(Self {
      i2c1,
      axp,
      imu: mpu6886,
      btn_a,
      btn_b,
      lcd: display,
      led,
    })
  }

  pub fn i2c1(&self) -> &Mutex<RefCell<I2cDriver<'a>>> {
    self.i2c1.as_ref()
  }

  pub fn update(&mut self) {
    self.btn_a.read();
    self.btn_b.read();
  }
}
