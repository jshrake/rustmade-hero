
use gfx;
use input;

fn render_weird_gradient(buffer: &mut gfx::PixelBuffer, x_offset: i32, y_offset: i32) {
  unsafe {
    let mut row = buffer.memory;
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

pub fn update_and_render(buffer: &mut gfx::PixelBuffer, input: &input::GameInput) {
  static mut x_offset: i32 = 0;
  static mut y_offset: i32 = 0;
  let p1 = &input.controllers[0];
  unsafe {
    println!("{} {}", p1.stick.x.stop, p1.stick.y.stop);
    x_offset += (10.0*p1.stick.x.stop) as i32;
    y_offset += (10.0*p1.stick.y.stop) as i32;
    render_weird_gradient(buffer, x_offset, y_offset);
  }
}
