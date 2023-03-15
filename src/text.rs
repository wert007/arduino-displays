use crate::display::Display;
use core::fmt::Debug;
mod ascii_font;

pub trait Writer<Color> {
    fn write_string(&mut self, text: &str, x: usize, y: usize, color: Color, bg_color: Color);
}

impl<Color: Copy + Debug, D: Display<Color>> Writer<Color> for D {
    fn write_string(&mut self, text: &str, mut x: usize, y: usize, color: Color, bg_color: Color) {
        let mut letter: [Color; 8 * 8];
        for byte in text.bytes() {
            letter = core::array::from_fn(|i| {
                let byte = ascii_font::BASIC_LEGACY[byte as usize][i / 8];
                if ((1 << (i % 8)) & byte) > 0 {
                    color
                } else {
                    bg_color
                }
            });
            self.set_frame_memory_from_buffer(&mut letter, x, y, 8, 8);
            x += 8;
        }
    }
}
