mod display_epaper;
mod display_oled;
mod display_oled_wide;

pub trait Display<Color: Copy + Debug> {
    const PIXEL_PER_BYTE: usize;
    /// The width of the display in pixel
    const WIDTH: usize;
    /// The height of the display in pixel
    const HEIGHT: usize;
    /// A light color in an unspecified color
    const LIGHT_COLOR: Color;
    /// A dark color most probably black
    const DARK_COLOR: Color;

    /// This calls the callback function for each pixel in the given area to
    /// fill it and overwrite the display in that area. The order in which this
    /// callback is called is not specified. It just has to be in an order,
    /// where it is the most performant or memory efficent.
    fn set_frame_memory_from_callback(
        &mut self,
        cb: impl Fn(usize, usize) -> Color,
        x: usize,
        y: usize,
        image_width: usize,
        image_height: usize,
    );

    /// This just calls the [`Display::set_frame_memory_from_callback`] function, with
    /// the buffer.
    fn set_frame_memory_from_buffer(
        &mut self,
        buffer: &mut [Color],
        x: usize,
        y: usize,
        image_width: usize,
        image_height: usize,
    ) {
        self.set_frame_memory_from_callback(|dx, dy| {
            let x = dx - x;
            let y = dy - y;
            buffer[y * image_width + x]
        }, x, y, image_width, image_height)
    }

    /// This just calls the [`Display::set_frame_memory_from_callback`]
    /// function, to clear the complete display to the specified color.
    fn clear_frame_memory(
        &mut self,
        clear_color: Color
    ) {
        self.set_frame_memory_from_callback(|_, _| clear_color, 0, 0, Self::WIDTH, Self::HEIGHT)
    }

    /// Depending on the display, this is needed to make the updated buffer
    /// actually visible. In other cases this is just a noop.
    fn display_frame(&mut self) {}

    /// The width of the display in pixel
    fn width(&self) -> usize { Self::WIDTH }
    /// The height of the display in pixel
    fn height(&self) -> usize { Self::HEIGHT }
    /// A light color in an unspecified color
    fn light_color(&self) -> Color { Self::LIGHT_COLOR }
    /// A dark color most probably black
    fn dark_color(&self) -> Color { Self::DARK_COLOR }
}

use core::fmt::Debug;

pub use display_epaper::EpaperDisplay;
pub use display_oled::OledDisplay;
pub use display_oled_wide::WideOledDisplay;
