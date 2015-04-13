#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate gdi32;
#[cfg(target_os = "windows")]
extern crate user32;

#[cfg(target_os = "windows")]
#[path="win32/mod.rs"]
mod platform;

mod game;
mod gfx;

fn main() {
  platform::main(game::update_and_render);
}
