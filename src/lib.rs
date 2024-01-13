#![no_std]
#![allow(clippy::result_unit_err)]
#![feature(decl_macro)]

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
use esp_idf_hal::i2c::I2C1;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use anyhow::Result;
use esp_idf_hal::spi::SPI3;
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

pub macro new_m5($peripherals:ident) {
  m5stickc::M5::new(M5Peripherals {
    i2c1: $peripherals.i2c1,
    spi3: $peripherals.spi3,
    gpio5: $peripherals.pins.gpio5,
    gpio10: $peripherals.pins.gpio10,
    gpio13: $peripherals.pins.gpio13,
    gpio15: $peripherals.pins.gpio15,
    gpio18: $peripherals.pins.gpio18,
    gpio21: $peripherals.pins.gpio21,
    gpio22: $peripherals.pins.gpio22,
    gpio23: $peripherals.pins.gpio23,
    gpio37: $peripherals.pins.gpio37,
    gpio39: $peripherals.pins.gpio39,
  })
}

pub struct M5Peripherals {
  // i2c1
  pub i2c1: I2C1,
  pub gpio21: Gpio21,
  pub gpio22: Gpio22,

  // spi3
  pub spi3: SPI3,
  pub gpio15: Gpio15,
  pub gpio13: Gpio13,
  pub gpio23: Gpio23,
  pub gpio5: Gpio5,
  pub gpio18: Gpio18,

  // LED
  pub gpio10: Gpio10,

  // Button A
  pub gpio37: Gpio37,
  // Button B
  pub gpio39: Gpio39,
}

pub struct M5<'a> {
  i2c1: Box<Mutex<RefCell<I2cDriver<'a>>>>,
  axp: axp192::Axp192<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>>,
  imu: mpu6886::MPU6886<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>>,
  btn_a: button::Button<PinDriver<'a, Gpio37, Input>>,
  btn_b: button::Button<PinDriver<'a, Gpio39, Input>>,
  lcd: Lcd<'a>,
  led: PinDriver<'a, Gpio10, Output>,
}

impl<'a> M5<'a> {
  pub fn new(peripherals: M5Peripherals) -> Result<Self, EspError> {
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c1 = I2cDriver::new(
      peripherals.i2c1,
      peripherals.gpio21,
      peripherals.gpio22,
      &config,
    )?;
    let i2c1 = Box::new(Mutex::new(RefCell::new(i2c1)));

    let i2c1_ref = unsafe { crate::misc::extend_lifetime(i2c1.as_ref()) };

    let axp = axp192::Axp192::new(i2c::CriticalSectionDevice::new(i2c1_ref)).unwrap();
    let mpu6886 = mpu6886::MPU6886::new(i2c::CriticalSectionDevice::new(i2c1_ref));

    let pin_a = PinDriver::input(peripherals.gpio37)?;
    let btn_a = button::Button::new(pin_a, true, 10);
    let pin_b = PinDriver::input(peripherals.gpio39)?;
    let btn_b = button::Button::new(pin_b, true, 10);

    let spi = peripherals.spi3;
    let tft_mosi = peripherals.gpio15;
    let tft_sclk = peripherals.gpio13;
    let tft_dc = PinDriver::output(peripherals.gpio23)?;
    let tft_cs = peripherals.gpio5;
    let tft_rst = PinDriver::output(peripherals.gpio18)?;

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

    let mut led = PinDriver::output(peripherals.gpio10)?;
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

  pub fn axp(&mut self) -> &mut axp192::Axp192<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>> {
    &mut self.axp
  }

  pub fn imu(&mut self) -> &mut mpu6886::MPU6886<i2c::CriticalSectionDevice<'a, I2cDriver<'a>>> {
    &mut self.imu
  }

  pub fn btn_a(&self) -> &button::Button<PinDriver<'a, Gpio37, Input>> {
    &self.btn_a
  }

  pub fn btn_b(&self) -> &button::Button<PinDriver<'a, Gpio39, Input>> {
    &self.btn_b
  }

  pub fn lcd(&mut self) -> &mut Lcd<'a> {
    &mut self.lcd
  }

  pub fn led(&mut self) -> &mut PinDriver<'a, Gpio10, Output> {
    &mut self.led
  }

  pub fn update(&mut self) {
    self.btn_a.read();
    self.btn_b.read();
  }
}
