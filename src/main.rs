#![no_std]
#![no_main]

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

  loop {
    info!(
      "AXP Temp: {:.1}C",
      m5.axp.get_temp_in_axp192().unwrap_or(f32::NAN)
    );
    info!(
      "[Bat] V: {:.3}v  I: {:.3}ma",
      m5.axp.get_bat_voltage().unwrap_or(f32::NAN),
      m5.axp.get_bat_current().unwrap_or(f32::NAN)
    );

    info!(
      "[USB]  V: {:.3}v  I: {:.3}ma",
      m5.axp.get_vbus_voltage().unwrap_or(f32::NAN),
      m5.axp.get_vbus_current().unwrap_or(f32::NAN)
    );

    delay.delay_ms(5000_u32).unwrap();
  }
}
