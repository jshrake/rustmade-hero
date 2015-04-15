use gfx;
use input;

pub type game_update_and_render = fn(buffer: &mut gfx::PixelBuffer, input: &input::GameInput);