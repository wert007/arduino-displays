use arduino_hal::{port, prelude::*, spi::{ChipSelectPin, self}};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use super::Display;

const EPAPER_SPI_SETTINGS: spi::Settings = spi::Settings {
    data_order: spi::DataOrder::MostSignificantFirst,
    clock: spi::SerialClockRate::OscfOver8,
    mode: embedded_hal::spi::MODE_0,
};

/// ```plain
/// Pin 1 = Power 5V
/// Pin 2 = GND
/// Pin 3 = MOSI
/// Pin 4 = SCK
/// Pin 5 = Chip Select
/// Pin 6 = Data Command
/// Pin 7 = Reset
/// Pin 8 = Busy
/// ```
pub struct EpaperDisplay<DCPin: OutputPin, BPin: InputPin, CSPin: port::PinOps> {
    data_command_pin: DCPin,
    busy_pin: BPin,
    chip_select_pin: ChipSelectPin<CSPin>,
    spi: arduino_hal::Spi,
}

#[allow(dead_code)]
impl<DCPin: OutputPin, BPin: InputPin, CSPin: port::PinOps> EpaperDisplay<DCPin, BPin, CSPin> {
    pub fn new(
        mut spi: arduino_hal::Spi,
        mut chip_select_pin: ChipSelectPin<CSPin>,
        mut reset_pin: impl OutputPin,
        data_command_pin: DCPin,
        busy_pin: BPin,
    ) -> Self {
        spi.reconfigure(EPAPER_SPI_SETTINGS).unwrap();

        arduino_hal::delay_ms(10);

        chip_select_pin.set_high().unwrap();

        let _ = reset_pin.set_low();
        arduino_hal::delay_ms(200);
        let _ = reset_pin.set_high();
        arduino_hal::delay_ms(200);

        // TODO: Software reset. The documentation says, this should be done,
        // but the c library does not do it..

        let mut result = EpaperDisplay {
            data_command_pin,
            spi,
            busy_pin,
            chip_select_pin,
        };

        result.send_command(0x01);
        result.send_data(&mut [0xC7, 0, 0]);

        result.send_command(0x11);
        result.send_data(&mut [0x03]);

        result.send_command(0x44);
        result.send_data(&mut [0x00, 0x18]);

        result.send_command(0x45);
        result.send_data(&mut [0xC7, 0, 0, 0]);

        result.send_command(0x3C);
        result.send_data(&mut [0x01]);

        // Until here the documentation and the library are very similiar. But
        // now they diverge. We'll try to stay close to the library at first and
        // test out other things later.

        result.send_command(0x21);
        result.send_data(&mut [0x00]);

        result.send_command(0x18);
        result.send_data(&mut [0x80]);

        result.send_command(0x22);
        result.send_data(&mut [0xB1]);

        result.send_command(0x20);

        result.block_until_idle();

        result
    }

    fn send_command(&mut self, command: u8) {
        self.data_command_pin
            .set_low()
            .unwrap_or_else(|_| todo!("Implement good panic"));
        self.chip_select_pin.set_low().unwrap();
        self.spi.transfer(&mut [command]).unwrap();
        self.chip_select_pin.set_high().unwrap();
    }

    fn send_data(&mut self, data: &mut [u8]) {
        self.data_command_pin
            .set_high()
            .unwrap_or_else(|_| todo!("Implement good panic"));
        self.chip_select_pin.set_low().unwrap();
        for word in data {
            self.spi.transfer(&mut [*word]).unwrap();
        }
        self.chip_select_pin.set_high().unwrap();
    }

    fn block_until_idle(&self) {
        while self
            .busy_pin
            .is_high()
            .unwrap_or_else(|_| todo!("Implement good panic!"))
        {
            arduino_hal::delay_ms(100);
        }
    }

    fn set_frame_memory_from_raw(
        &mut self,
        image_buffer: &mut [u8],
        mut x: usize,
        y: usize,
        mut image_width: usize,
        image_height: usize,
    ) {
        x &= 0xF8;
        image_width &= 0xF8;

        let x_end = if x + image_width >= Self::WIDTH {
            Self::WIDTH - 1
        } else {
            x + image_width - 1
        };
        let y_end = if y + image_height >= Self::HEIGHT {
            Self::HEIGHT - 1
        } else {
            y + image_height - 1
        };

        self.set_memory_area(x, y, x_end, y_end);

        for line in y..=y_end {
            self.set_memory_pointer(x, line);
            self.send_command(0x24);
            self.send_data(
                &mut image_buffer[(line - y) * image_width / 8..(line - y + 1) * image_width / 8],
            );
        }
    }

    fn set_memory_area(&mut self, x: usize, y: usize, x_end: usize, y_end: usize) {
        self.send_command(0x44);
        self.send_data(&mut [(x >> 3) as u8, (x_end >> 3) as u8]);
        self.send_command(0x45);
        self.send_data(&mut [y as u8, (y >> 8) as u8, y_end as u8, (y_end >> 8) as u8]);
    }

    fn set_memory_pointer(&mut self, x: usize, y: usize) {
        self.send_command(0x4E);
        self.send_data(&mut [(x >> 3) as u8]);
        self.send_command(0x4F);
        self.send_data(&mut [y as u8, (y >> 8) as u8]);
        self.block_until_idle();
    }
}

impl<DCPin: OutputPin, BPin: InputPin, CSPin: port::PinOps> Display<bool>
    for EpaperDisplay<DCPin, BPin, CSPin>
{
    fn set_frame_memory_from_callback(
        &mut self,
        f: impl Fn(usize, usize) -> bool,
        mut x: usize,
        y: usize,
        mut image_width: usize,
        image_height: usize,
    ) {
        x &= 0xF8;
        image_width &= 0xF8;

        let x_end = if x + image_width >= Self::WIDTH {
            Self::WIDTH - 1
        } else {
            x + image_width - 1
        };
        let y_end = if y + image_height >= Self::HEIGHT {
            Self::HEIGHT - 1
        } else {
            y + image_height - 1
        };

        self.set_memory_area(x, y, x_end, y_end);

        for line in y..=y_end {
            self.set_memory_pointer(x, line);
            self.send_command(0x24);
            for cursor in (x..=x_end).step_by(8) {
                let mut byte = 0;
                for pixel_x in 0..8 {
                    byte <<= 1;
                    byte |= f(cursor + pixel_x, line) as u8;
                }
                self.send_data(&mut [byte]);
            }
        }
    }

    fn display_frame(&mut self) {
        self.send_command(0x22);
        self.send_data(&mut [0xC7]);
        self.send_command(0x20);
        self.block_until_idle();
    }

    const PIXEL_PER_BYTE: usize = 8;
    const HEIGHT: usize = 200;
    const WIDTH: usize = 200;
    const DARK_COLOR: bool = false;
    const LIGHT_COLOR: bool = true;
}
