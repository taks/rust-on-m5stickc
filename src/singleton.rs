macro_rules! singleton {
  (: $ty:ty = $expr:expr) => {
    esp_idf_hal::interrupt::free(|| {
      static mut VAR: Option<$ty> = None;

      #[allow(unsafe_code)]
      let used = unsafe { VAR.is_some() };
      if used {
        None
      } else {
        let expr = $expr;

        #[allow(unsafe_code)]
        unsafe {
          VAR = Some(expr)
        }

        #[allow(unsafe_code)]
        unsafe {
          VAR.as_mut()
        }
      }
    })
  };
}
pub(crate) use singleton;