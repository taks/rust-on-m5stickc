
pub fn millis() -> u32 {
  unsafe { (esp_idf_sys::esp_timer_get_time() / 1000) as u32 }
}

pub fn map(x: i64, in_min: i64, in_max: i64, out_min: i64, out_max: i64) -> i64 {
  let run = in_max - in_min;
  let rise = out_max - out_min;
  let delta = x - in_min;
  return (delta * rise) / run + out_min;
}