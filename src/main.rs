#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;

use std::ptr;
use std::mem;

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
  use std::ptr;
  use std::mem;
  use winapi;
  use user32;
  use kernel32;

  pub fn message_box(owner : winapi::HWND, message : &str,
                     title : &str, typ: u32) {
      unsafe {
          user32::MessageBoxW(
              owner,
              utf16!(message).as_ptr(),
              utf16!(title).as_ptr(),
              typ);   
      }
  }


  pub unsafe fn register_window(name :&str, callback :winapi::WNDPROC) -> Vec<u16> {
    let win_class_name = utf16!(name);
    unsafe {
      let win_class = winapi::WNDCLASSEXW{
         cbSize: mem::size_of::<winapi::WNDCLASSEXW>() as winapi::UINT,
         style: winapi::CS_OWNDC | winapi::CS_HREDRAW | winapi::CS_VREDRAW as winapi::UINT,
         lpfnWndProc: callback,
         cbClsExtra: 0,
         cbWndExtra: 0,
         hInstance:  kernel32::GetModuleHandleW(ptr::null()),
         hIcon: ptr::null_mut(),
         hCursor: ptr::null_mut(),
         hbrBackground: ptr::null_mut(),
         lpszMenuName: ptr::null(),
         lpszClassName: win_class_name.as_ptr(),
         hIconSm: ptr::null_mut()
      };
      user32::RegisterClassExW(&win_class);
    }
    win_class_name
  }

  pub unsafe fn create_window(win_class_name :&Vec<u16>) -> winapi::HWND {
    let handle = user32::CreateWindowExW(
      0,
      win_class_name.as_ptr(),
      utf16!("Rustmade Hero!").as_ptr(),
      winapi::WS_OVERLAPPEDWINDOW | winapi::WS_VISIBLE,
      winapi::CW_USEDEFAULT,
      winapi::CW_USEDEFAULT,
      winapi::CW_USEDEFAULT,
      winapi::CW_USEDEFAULT,
      ptr::null_mut(),
      ptr::null_mut(),
      kernel32::GetModuleHandleW(ptr::null()),
      ptr::null_mut());
    debug_assert!(handle != ptr::null_mut(), "user32::CreateWindowExW failed");
    loop {
      let mut msg = mem::zeroed();
      if user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) <= 0 {
        break;
      }
      user32::TranslateMessage(&msg);
      user32::DispatchMessageW(&msg);
    };
    return handle;
  }

}


/*
LRESULT CALLBACK WindowProc(
  _In_  HWND hwnd,
  _In_  UINT uMsg,
  _In_  WPARAM wParam,
  _In_  LPARAM lParam
);
*/
pub unsafe extern "system" fn callback(window: winapi::HWND,
                                       msg: winapi::UINT,
                                       wparam: winapi::WPARAM,
                                       lparam: winapi::LPARAM)
                                       -> winapi::LRESULT
{
  match msg {
    winapi::WM_ACTIVATEAPP => {
      0
    },
    winapi::WM_CLOSE => {
      0
    },
    winapi::WM_DESTROY => {
      0
    },
    winapi::WM_SIZE => {
      0
    },
    _ => user32::DefWindowProcW(window, msg, wparam, lparam)
  }
}

fn main() {
  unsafe {
    let name = windows::register_window("WinClass", Some(callback));
    windows::create_window(&name);
  }

}
