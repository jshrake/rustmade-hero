use std::ptr;
use std::default::Default;

pub struct PixelBuffer {
  pub memory: *mut u8,
  pub width: i32,
  pub height: i32,
  pub pitch: i32
}

impl Default for PixelBuffer {
  fn default() -> PixelBuffer {
    PixelBuffer {
      memory: ptr::null_mut(),
      width: 0,
      height: 0,
      pitch: 0
    }
  }
}