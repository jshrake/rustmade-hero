#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;

use std::ptr;

macro_rules! utf16 {
	($s:expr) => {
		{
		 use std::ffi::{OsStr};
		 use std::os::windows::ffi::OsStrExt;
		 OsStr::new($s).
			 encode_wide().
			 chain(Some(0).into_iter()).
			 collect::<Vec<_>>()
		}
	}
}

mod windows {
	use winapi;
	use user32;
	pub fn message_box(owner : winapi::HWND, message : &str, title : &str, typ: u32) {
		unsafe {
			user32::MessageBoxW(
				owner,
				utf16!(message).as_ptr(),
				utf16!(title).as_ptr(),
				typ);	
		}
	}
}

fn main() {
	windows::message_box(ptr::null_mut(),
		"¥ · £ · € · $ · ¢ · ₡ · ₢ · ₣ · ₤ · ₥ · ₦ · ₧ · ₨ · ₩ · ₪ · ₫ · ₭ · ₮ · ₯ · ₹",
		"It's a MessageBoxW!",
		winapi::MB_OK | winapi::MB_ICONINFORMATION);
}
