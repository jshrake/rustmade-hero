use std::ptr;
use std::mem;
use winapi::*;
use user32;
use kernel32;
use gdi32;
use std::default::Default;
use gfx;

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

fn XInputGetStateStub(dwUserIndex: DWORD , pState: *mut XINPUT_STATE) -> DWORD {
  ERROR_DEVICE_NOT_CONNECTED
}
static mut XInputGetState: fn(DWORD, *mut XINPUT_STATE) -> DWORD = XInputGetStateStub;
fn XInputSetStateStub(dwUserIndex: DWORD , pVibration: *mut XINPUT_VIBRATION) -> DWORD {
  ERROR_DEVICE_NOT_CONNECTED
}
static mut XInputSetState: fn(DWORD, *mut XINPUT_VIBRATION) -> DWORD = XInputSetStateStub;

fn load_xinput_lib() {
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

static DEFAULT_BITMAP_INFO: BITMAPINFOHEADER = BITMAPINFOHEADER {
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
  pub info: BITMAPINFO<'a>,
  pub buffer: gfx::PixelBuffer
}

impl <'a> Default for OffscreenBuffer<'a> {
  fn default() -> OffscreenBuffer<'a> {
    OffscreenBuffer {
      info: BITMAPINFO {
          bmiHeader: DEFAULT_BITMAP_INFO,
          bmiColors: &[]
      },
      buffer: gfx::PixelBuffer::default()
    }
  }
}

impl <'a> OffscreenBuffer <'a> {
  fn new(width: i32, height: i32) -> OffscreenBuffer<'a> {
    let mut osb = OffscreenBuffer::default();
    osb.buffer.width = width;
    osb.buffer.height = height;
    let bytes_per_pixel = 4;
    osb.buffer.pitch = width * bytes_per_pixel;
    osb.info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    osb.info.bmiHeader.biWidth = width;
    osb.info.bmiHeader.biHeight = height;
    osb.info.bmiHeader.biBitCount = 32;
    osb.info.bmiHeader.biCompression = BI_RGB;

    let bitmap_mem_size = (width * height * bytes_per_pixel) as SIZE_T;
    unsafe {
      osb.buffer.memory = kernel32::VirtualAlloc(ptr::null_mut(),
        bitmap_mem_size as SIZE_T, MEM_COMMIT,PAGE_READWRITE) as *mut u8;
    }
    osb
  }
}

fn register_window(name :&str, callback :WNDPROC) -> Vec<u16> {
  let win_class_name = utf16!(name);
  unsafe {
    let win_class = WNDCLASSEXW{
       cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
       style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW as UINT,
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

fn display_buffer_in_window(device_context :HDC,
                            window_width: i32,
                            window_height: i32,
                            backbuffer: &mut OffscreenBuffer) {
  unsafe {
  gdi32::StretchDIBits(device_context,
                       0, 0, window_width, window_height,
                       0, 0, backbuffer.buffer.width, backbuffer.buffer.height,
                       backbuffer.buffer.memory as LPVOID,
                       &backbuffer.info,
                       DIB_RGB_COLORS, SRCCOPY);
  }
}

fn create_window(win_class_name :&Vec<u16>, game_update_and_render: fn(&mut gfx::PixelBuffer, i32, i32)) -> HWND {
  let mut backbuffer = OffscreenBuffer::new(1280, 720);
  unsafe {
    let window = user32::CreateWindowExW(
      0,
      win_class_name.as_ptr(),
      utf16!("Rustmade Hero!").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
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
      while user32::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) > 0 {
        if msg.message == WM_QUIT {
          running = false;
        }
        user32::TranslateMessage(&msg);
        user32::DispatchMessageW(&msg);
      }
      //todo(jshrake): Should we poll this more frequently
      for controller_index in 0..XUSER_MAX_COUNT {
        let mut controller_state = mem::zeroed();
        if XInputGetState(controller_index, &mut controller_state) == ERROR_SUCCESS {
          //note(jshrake): Controller is plugged in
          let pad = &controller_state.Gamepad;
          let up = pad.wButtons & XINPUT_GAMEPAD_DPAD_UP;
          let down = pad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN;
          let left = pad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT;
          let right = pad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT;
          let start = pad.wButtons & XINPUT_GAMEPAD_START;
          let back = pad.wButtons & XINPUT_GAMEPAD_BACK;
          let left_shoulder = pad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER;
          let right_shoulder = pad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER;
          let a_button = pad.wButtons & XINPUT_GAMEPAD_A;
          let b_button = pad.wButtons & XINPUT_GAMEPAD_B;
          let x_button = pad.wButtons & XINPUT_GAMEPAD_X;
          let y_button = pad.wButtons & XINPUT_GAMEPAD_Y;
          let stick_x = pad.sThumbLX as i32;
          let stick_y = pad.sThumbLY as i32;
          x_offset += stick_x >> 12;
          y_offset += stick_y >> 12;
          let mut vibration = XINPUT_VIBRATION {
            wLeftMotorSpeed: 60000,
            wRightMotorSpeed: 60000
          };
          XInputSetState(controller_index, &mut vibration);
        } else {
          //note(jshrake): Controller is not plugged in
        }
      }
      game_update_and_render(&mut backbuffer.buffer, x_offset, y_offset);
      let mut client_rect = mem::zeroed();
      user32::GetClientRect(window, &mut client_rect);
      let client_width = client_rect.right - client_rect.left;
      let client_height = client_rect.bottom - client_rect.top;
      display_buffer_in_window(device_context, client_width, client_height, &mut backbuffer);

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

pub unsafe extern "system" fn callback(window: HWND,
                                     msg: UINT,
                                     wparam: WPARAM,
                                     lparam: LPARAM)
                                     -> LRESULT
{
match msg {
  WM_ACTIVATEAPP => {
    0
  },
  WM_CLOSE => {
    user32::PostQuitMessage(0);
    0
  },
  WM_DESTROY => {
    user32::PostQuitMessage(0);
    0
  },
  WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
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
        VK_UP => {
          println!("UP");
        },
        VK_LEFT => {
          println!("LEFT");
        },
        VK_DOWN => {
          println!("DOWN");
        },
        VK_RIGHT => {
          println!("RIGHT");
        },
        VK_ESCAPE => {
          println!("ESCAPE");
          user32::PostQuitMessage(0);
        },
        VK_SPACE => {
          println!("SPACE");
        },
        _ => {}
      }
    }
    0
  },
  WM_SIZE => {
    0
  },

  _ => user32::DefWindowProcW(window, msg, wparam, lparam)
  }
}


pub fn main(game_update_and_render: fn(&mut gfx::PixelBuffer, i32, i32)) {
  load_xinput_lib();
  let name = register_window("WinClass", Some(callback));
  create_window(&name, game_update_and_render);
}