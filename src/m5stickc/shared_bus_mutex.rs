#[macro_export]
macro_rules! new {
  ($bus_type:ty = $bus:expr) => {{
      let m: Option<&'static mut _> = singleton::singleton!(
          : shared_bus_mutex::BusManager<$bus_type> =
          shared_bus_mutex::BusManager::new($bus)
      );

      m
  }};
}
pub(crate) use new;

pub struct SharedBusMutex<T>(esp_idf_hal::mutex::Mutex<T>);

impl<T> shared_bus::BusMutex for SharedBusMutex<T> {
  type Bus = T;

  fn create(v: T) -> Self {
    Self(esp_idf_hal::mutex::Mutex::new(v))
  }

  fn lock<R, F: FnOnce(&mut Self::Bus) -> R>(&self, f: F) -> R {
    let mut v = self.0.lock();
    f(&mut v)
  }
}

pub type BusManager<BUS> = shared_bus::BusManager<SharedBusMutex<BUS>>;