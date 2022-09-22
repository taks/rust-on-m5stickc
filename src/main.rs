#![no_std]
#![no_main]

pub mod m5stickc;

use log::*;

#[no_mangle]
fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  let result = m5stickc::axp192::begin(m5stickc::axp192::BeginConf::default());
  if result.is_err() {
    error!("axp192: Initialization failure");
  }

  info!("Hello, world!");
}
