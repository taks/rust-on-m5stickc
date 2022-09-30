#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use core::fmt::Write;

use embedded_graphics::{
  pixelcolor::Rgb565,
  prelude::{Point, RgbColor},
};
use embedded_hal::delay::blocking::DelayUs;
use m5stickc::display_buffer;

#[no_mangle]
fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  // WDT OFF
  unsafe {
    esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
      esp_idf_hal::cpu::core() as u32,
    ));
  };

  let mut delay = esp_idf_hal::delay::Ets {};
  let mut m5 = m5stickc::M5::new().unwrap();
  m5.mpu6886.init().unwrap();

  let mut canvas = display_buffer::DisplayBuffer::new(
    Rgb565::BLACK,
    Rgb565::WHITE,
    m5.lcd.width(),
    m5.lcd.height(),
  );

  loop {
    m5.update();
    canvas.clear_default();
    canvas.cursur = Point::new(0, 0);

    let gyro_result = m5.mpu6886.get_gyro_data();
    let accel_result = m5.mpu6886.get_accel_data();

    if gyro_result.is_ok() && accel_result.is_ok() {
      let (gyro_x, gyro_y, gyro_z) = gyro_result.unwrap();
      let (acc_x, acc_y, acc_z) = accel_result.unwrap();

      writeln!(canvas, "  X       Y       Z").unwrap();
      writeln!(
        canvas,
        "{:.2}   {:.2}   {:.2}      o/s",
        gyro_x, gyro_y, gyro_z
      )
      .unwrap();
      writeln!(canvas, "{:.2}   {:.2}   {:.2}", acc_x, acc_y, acc_z).unwrap();
    } else {
      write!(canvas, "Sensor read error").unwrap();
    }

    m5.lcd.draw(&mut canvas).unwrap();
    delay.delay_ms(100).unwrap();
  }
}
