use super::misc::millis;

pub struct Button<PIN: embedded_hal::digital::blocking::InputPin> {
  pin: PIN,
  state: bool,     // current button state
  invert: bool,    // if false, interpret high state as pressed, else interpret low state as pressed
  last_state: bool, // previous button state
  changed: bool,   // state changed since last read
  time: u32,       // time of current state (all times are in ms)
  last_time: u32,   // time of previous state
  last_change: u32, // time of last state change
  db_time: u32,     // debounce time
  press_time: u32,  // press time
}

impl<PIN> Button<PIN>
where
  PIN: embedded_hal::digital::blocking::InputPin,
{
  pub fn new(pin: PIN, invert: bool, db_time: u32) -> Self {
    let mut state = pin.is_high().unwrap();
    if invert {
      state = !state;
    }

    let time = millis();
    Button {
      pin,
      state,
      invert,
      last_state: state,
      time,
      last_time: time,
      changed: false,
      last_change: time,
      db_time,
      press_time: time,
    }
  }

  pub fn read(&mut self) -> bool {
    let ms = millis();

    self.last_time = self.time;
    self.time = ms;
    if ms - self.last_change < self.db_time {
      self.changed = false;
    } else {
      self.last_state = self.state;
      self.state = self.pin.is_high().unwrap();
      if self.invert {
        self.state = !self.state;
      }
      self.changed = self.state != self.last_state;
      if self.changed {
        self.last_change = ms;
        self.changed = true;
        if self.state {
          self.press_time = self.time;
        }
      }
    }
    self.state
  }

  pub fn pressed(&mut self) -> bool {
    self.state
  }

}
