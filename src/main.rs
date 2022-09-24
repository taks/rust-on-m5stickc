#![no_std]
#![no_main]

extern crate alloc;

pub mod m5stickc;

use embedded_hal::delay::blocking::DelayUs;
use log::*;

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
    ))
  };

  let mut delay = esp_idf_hal::delay::Ets {};
  let mut m5 = m5stickc::M5::new().unwrap();
  m5.mpu6886.init().unwrap();

  loop {
    m5.update();
    let gyro_result = m5.mpu6886.get_gyro_data();
    let accel_result = m5.mpu6886.get_accel_data();

    if gyro_result.is_ok() && accel_result.is_ok() {
      let (gyro_x, gyro_y, gyro_z) = gyro_result.unwrap();
      let (acc_x, acc_y, acc_z) = accel_result.unwrap();
      info!(
        "{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}",
        gyro_x, gyro_y, gyro_z, acc_x * 1000.0, acc_y * 1000.0, acc_z * 1000.0
      );
    }

    delay.delay_ms(50_u32).unwrap();
  }
}
