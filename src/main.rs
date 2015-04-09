#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate xinput9_1_0;

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
  use gdi32;
  use xinput9_1_0;
  use std::default::Default;

  static DEFAULT_BITMAP_INFO: winapi::BITMAPINFOHEADER = winapi::BITMAPINFOHEADER {
        biSize: 0,
        biWidth: 0,
        biHeight: 0,
        biPlanes: 1,
        biBitCount: 0,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0 
      };


  pub struct OffscreenBuffer<'a> {
    pub info: winapi::BITMAPINFO<'a>,
    pub memory: winapi::PVOID,
    pub width: i32,
    pub height: i32,
    pub pitch: i32
  }

  impl <'a> Default for OffscreenBuffer<'a> {
    fn default() -> OffscreenBuffer<'a> {
      OffscreenBuffer {
        info: winapi::BITMAPINFO {
            bmiHeader: DEFAULT_BITMAP_INFO,
            bmiColors: &[]
        },
        memory: ptr::null_mut(),
        width: 0,
        height: 0,
        pitch: 0
      }
    }
  }

  impl <'a> OffscreenBuffer <'a> {
    pub fn new(width: i32, height: i32) -> OffscreenBuffer<'a> {
      let mut osb = OffscreenBuffer::default();
      osb.width = width;
      osb.height = height;
      let bytes_per_pixel = 4;
      osb.pitch = width * bytes_per_pixel;
      osb.info.bmiHeader.biSize = mem::size_of::<winapi::BITMAPINFOHEADER>() as u32;
      osb.info.bmiHeader.biWidth = width;
      osb.info.bmiHeader.biHeight = height;
      osb.info.bmiHeader.biBitCount = 32;
      osb.info.bmiHeader.biCompression = winapi::BI_RGB;

      let bitmap_mem_size = (width * height * bytes_per_pixel) as winapi::SIZE_T;
      unsafe {
        osb.memory = kernel32::VirtualAlloc(ptr::null_mut(),
          bitmap_mem_size, winapi::MEM_COMMIT, winapi::PAGE_READWRITE);
      }
      osb
    }
  }

  pub fn register_window(name :&str, callback :winapi::WNDPROC) -> Vec<u16> {
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

  fn display_buffer_in_window(device_context :winapi::HDC,
                              window_width: i32,
                              window_height: i32,
                              backbuffer: &OffscreenBuffer) {
    unsafe {
    gdi32::StretchDIBits(device_context,
                         0, 0, window_width, window_height,
                         0, 0, backbuffer.width, backbuffer.height,
                         backbuffer.memory,
                         &backbuffer.info,
                         winapi::DIB_RGB_COLORS, winapi::SRCCOPY);
    }
  }

  fn render_weird_gradient(buffer: &mut OffscreenBuffer, x_offset: i32, y_offset: i32) {
    unsafe {
      let mut row = buffer.memory as *mut u8;
      for y in 0..buffer.height {
        let mut pixel = row as *mut u32;
        for x in 0..buffer.width {
          let blue = ((x + x_offset) as u8) as u32;
          let green = ((y + y_offset) as u8) as u32;
          *pixel = (green << 8) | blue;
          pixel = pixel.offset(1);
        }
        row = row.offset(buffer.pitch as isize);
      }
    }
  }

  pub fn create_window(win_class_name :&Vec<u16>, backbuffer: &mut OffscreenBuffer) -> winapi::HWND {
    unsafe {
      let window = user32::CreateWindowExW(
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
      debug_assert!(window != ptr::null_mut(), "user32::CreateWindowExW failed");
      let device_context = user32::GetDC(window);
      let mut x_offset = 0;
      let mut y_offset = 0;
      let mut running = true;
      while running {
        let mut msg = mem::zeroed();
        while user32::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, winapi::PM_REMOVE) > 0 {
          if msg.message == winapi::WM_QUIT {
            running = false;
          }
          user32::TranslateMessage(&msg);
          user32::DispatchMessageW(&msg);
        }
        //todo(jshrake): Should we poll this more frequently
        for controller_index in 0..winapi::XUSER_MAX_COUNT {
          let mut controller_state = mem::zeroed();
          if xinput9_1_0::XInputGetState(controller_index, &mut controller_state) == winapi::ERROR_SUCCESS {
            //note(jshrake): Controller is plugged in
            let pad = &controller_state.Gamepad;
            let up = pad.wButtons & winapi::XINPUT_GAMEPAD_DPAD_UP;
            let down = pad.wButtons & winapi::XINPUT_GAMEPAD_DPAD_DOWN;
            let left = pad.wButtons & winapi::XINPUT_GAMEPAD_DPAD_LEFT;
            let right = pad.wButtons & winapi::XINPUT_GAMEPAD_DPAD_RIGHT;
            let start = pad.wButtons & winapi::XINPUT_GAMEPAD_START;
            let back = pad.wButtons & winapi::XINPUT_GAMEPAD_BACK;
            let left_shoulder = pad.wButtons & winapi::XINPUT_GAMEPAD_LEFT_SHOULDER;
            let right_shoulder = pad.wButtons & winapi::XINPUT_GAMEPAD_RIGHT_SHOULDER;
            let a_button = pad.wButtons & winapi::XINPUT_GAMEPAD_A;
            let b_button = pad.wButtons & winapi::XINPUT_GAMEPAD_B;
            let x_button = pad.wButtons & winapi::XINPUT_GAMEPAD_X;
            let y_button = pad.wButtons & winapi::XINPUT_GAMEPAD_Y;
            let stick_x = pad.sThumbLX as i32;
            let stick_y = pad.sThumbLY as i32;
            x_offset += stick_x >> 12;
            y_offset += stick_y >> 12;
          } else {
            //note(jshrake): Controller is not plugged in
          }
        }
        render_weird_gradient(backbuffer, x_offset, y_offset);
        let mut client_rect = mem::zeroed();
        user32::GetClientRect(window, &mut client_rect);
        let client_width = client_rect.right - client_rect.left;
        let client_height = client_rect.bottom - client_rect.top;
        display_buffer_in_window(device_context, client_width, client_height, backbuffer);
      };
      window
    }
  }

}

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
      user32::PostQuitMessage(0);
      0
    },
    winapi::WM_DESTROY => {
      user32::PostQuitMessage(0);
      0
    },
    winapi::WM_SIZE => {
      0
    },
    _ => user32::DefWindowProcW(window, msg, wparam, lparam)
  }
}

fn main() {
  let name = windows::register_window("WinClass", Some(callback));
  let mut backbuffer = windows::OffscreenBuffer::new(1920, 1080);
  windows::create_window(&name, &mut backbuffer);
}
