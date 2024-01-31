#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::Write;

use embedded_graphics::{
  geometry::OriginDimensions,
  pixelcolor::Rgb565,
  prelude::{Point, RgbColor},
};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::timer::EspTaskTimerService;
use m5stickc::display_buffer;

#[no_mangle]
fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();

  let mut m5 = m5stickc::new_m5!(peripherals).unwrap();
  m5.imu().init().unwrap();

  let mut canvas = display_buffer::DisplayBuffer::new(
    Rgb565::BLACK,
    Rgb565::WHITE,
    m5.lcd().size().width as _,
    m5.lcd().size().height as _,
  );

  let timer = EspTaskTimerService::new().unwrap();
  let mut prev = timer.now();

  loop {
    m5.update();
    canvas.clear_default();
    canvas.cursur = Point::new(0, 0);

    let now = timer.now();
    let fps = 1.0 / (now - prev).as_secs_f32();
    writeln!(canvas, "fps: {:.2}", fps).unwrap();
    prev = now;

    let gyro_result = m5.imu().get_gyro_data();
    let accel_result = m5.imu().get_accel_data();

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

    m5.draw(&canvas).unwrap();
    esp_idf_hal::delay::FreeRtos::delay_ms(100);
  }
}
