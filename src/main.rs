#![feature(asm)]
#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;

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
  use winapi::*;
  use user32;
  use kernel32;
  use gdi32;
  use std::default::Default;

  fn XInputGetStateStub(dwUserIndex: winapi::DWORD , pState: *mut winapi::XINPUT_STATE) -> winapi::DWORD {
    winapi::ERROR_DEVICE_NOT_CONNECTED
  }
  static mut XInputGetState: fn(winapi::DWORD, *mut winapi::XINPUT_STATE) -> winapi::DWORD = XInputGetStateStub;
  fn XInputSetStateStub(dwUserIndex: winapi::DWORD , pVibration: *mut winapi::XINPUT_VIBRATION) -> winapi::DWORD {
    winapi::ERROR_DEVICE_NOT_CONNECTED
  }
  static mut XInputSetState: fn(winapi::DWORD, *mut winapi::XINPUT_VIBRATION) -> winapi::DWORD = XInputSetStateStub;

  pub fn load_xinput_lib() {
    unsafe {
      let name_1_4 = utf16!("xinput1_4.dll");
      let mut xinput_lib = kernel32::LoadLibraryW(name_1_4.as_ptr());
      if xinput_lib == ptr::null_mut() {
        let name_1_3 = utf16!("xinput1_3.dll");
        xinput_lib = kernel32::LoadLibraryW(name_1_3.as_ptr());
        println!("Found xinput1_3");
      } else {
        println!("Found xinput1_4");
      }

      if xinput_lib != ptr::null_mut() {
        // in release builds, the optimizer is stomping on xinput_[g, s]et_state_addr. assert to the rescue
        let xinput_get_state_addr =  kernel32::GetProcAddress(xinput_lib, "XInputGetState".as_ptr() as *const i8);
        assert!(xinput_get_state_addr != ptr::null_mut(), "Couldn't find XInputGetState");
        XInputGetState = mem::transmute(xinput_get_state_addr);
        let xinput_set_state_addr =  kernel32::GetProcAddress(xinput_lib, "XInputSetState".as_ptr() as *const i8);
        assert!(xinput_set_state_addr != ptr::null_mut(), "Couldn't find XInputSetState");
        XInputSetState = mem::transmute(xinput_set_state_addr);
      } else {
        println!("xinput not found!");
      }
    }
  }

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

      let mut last_counter = mem::zeroed();
      kernel32::QueryPerformanceCounter(&mut last_counter);
      let mut perf_count_frequency = mem::zeroed();
      kernel32::QueryPerformanceFrequency(&mut perf_count_frequency);
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
          if XInputGetState(controller_index, &mut controller_state) == winapi::ERROR_SUCCESS {
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
            let mut vibration = winapi::XINPUT_VIBRATION {
              wLeftMotorSpeed: 60000,
              wRightMotorSpeed: 60000
            };
            XInputSetState(controller_index, &mut vibration);
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

        let mut end_counter = mem::zeroed();
        kernel32::QueryPerformanceCounter(&mut end_counter);
        let counter_elapsed = (end_counter - last_counter) as f32;
        let ms_per_frame = (1000.0 * counter_elapsed) / perf_count_frequency as f32;
        println!("{}ms/f, {}f/s", ms_per_frame, 1000.0 / ms_per_frame);

        last_counter = end_counter;
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
    winapi::WM_SYSKEYDOWN | winapi::WM_SYSKEYUP | winapi::WM_KEYDOWN | winapi::WM_KEYUP => {
      let vkcode = wparam as u8 as char;
      // 30th bit in lparam determines if the key was up or down
      let was_down = (lparam & (1 << 30)) != 0;
      // 31st bit in lparam determines if the key is up or down
      let is_down = (lparam & (1 << 31)) == 0;
      if was_down != is_down {
        match vkcode {
          'W' => {
            println!("W");
          },
          'A' => {
            println!("A");
          },
          'S' => {
            println!("S");
          },
          'D' => {
            println!("D");
          },
          'Q' => {
            println!("Q");
          },
          'E' => {
            println!("E");
          },
          _ => {}
        }
        match wparam {
          winapi::VK_UP => {
            println!("UP");
          },
          winapi::VK_LEFT => {
            println!("LEFT");
          },
          winapi::VK_DOWN => {
            println!("DOWN");
          },
          winapi::VK_RIGHT => {
            println!("RIGHT");
          },
          winapi::VK_ESCAPE => {
            println!("ESCAPE");
            user32::PostQuitMessage(0);
          },
          winapi::VK_SPACE => {
            println!("SPACE");
          },
          _ => {}
        }
      }
      0
    },
    winapi::WM_SIZE => {
      0
    },

    _ => user32::DefWindowProcW(window, msg, wparam, lparam)
  }
}

fn main() {
  windows::load_xinput_lib();
  let name = windows::register_window("WinClass", Some(callback));
  let mut backbuffer = windows::OffscreenBuffer::new(1280, 720);
  windows::create_window(&name, &mut backbuffer);
}
