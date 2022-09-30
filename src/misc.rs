
pub fn millis() -> u32 {
  unsafe { (esp_idf_sys::esp_timer_get_time() / 1000) as u32 }
}

pub fn map(x: i64, in_min: i64, in_max: i64, out_min: i64, out_max: i64) -> i64 {
  let run = in_max - in_min;
  let rise = out_max - out_min;
  let delta = x - in_min;
  (delta * rise) / run + out_min
}

#[inline(always)]
#[allow(dead_code)]
pub fn as_mut_slice_of<'a, FROM, TO>(src: &'a [FROM]) -> &'a [TO] {
  let size_of_from = core::mem::size_of::<FROM>();
  let size_of_to = core::mem::size_of::<TO>();
  unsafe { core::slice::from_raw_parts(src.as_ptr() as *mut TO, src.len() * size_of_from / size_of_to) }
}

pub fn enable_core0_wdt() {
  todo!()
}
pub fn disable_core0_wdt() {
  todo!()
}