use arduino_hal::{
    port,
    prelude::_embedded_hal_blocking_spi_Transfer,
    spi::{self, ChipSelectPin},
};
use embedded_hal::digital::v2::OutputPin;

use super::Display;

const OLED_SPI_SETTINGS: spi::Settings = spi::Settings {
    data_order: spi::DataOrder::MostSignificantFirst,
    clock: spi::SerialClockRate::OscfOver128,
    mode: embedded_hal::spi::MODE_0,
};

pub struct WideOledDisplay<DCPin: OutputPin, CSPin: port::PinOps> {
    data_command_pin: DCPin,
    chip_select_pin: ChipSelectPin<CSPin>,
    spi: arduino_hal::Spi,
}

#[allow(dead_code)]
impl<DCPin: OutputPin, CSPin: port::PinOps> WideOledDisplay<DCPin, CSPin> {
    pub fn new(
        mut spi: arduino_hal::Spi,
        mut chip_select_pin: ChipSelectPin<CSPin>,
        mut reset_pin: impl OutputPin,
        data_command_pin: DCPin,
    ) -> Self {
        spi.reconfigure(OLED_SPI_SETTINGS).unwrap();

        chip_select_pin.set_low().unwrap();
        let _ = reset_pin.set_high();
        arduino_hal::delay_ms(10);
        let _ = reset_pin.set_low();
        arduino_hal::delay_ms(10);
        let _ = reset_pin.set_high();

        let mut result = WideOledDisplay {
            data_command_pin,
            spi,
            chip_select_pin,
        };

        result.send_command(&mut [
            0x20,
            0x00, // -- Set horizontal addressing mode
            0xa1, // -- Turn Display upside down
            0xc8, // -- Flip Display horizontally
            0xaf, // -- turn on oled panel
        ]);
        result
    }

    fn send_command(&mut self, commands: &mut [u8]) {
        let _ = self.data_command_pin.set_low();
        self.chip_select_pin.set_low().unwrap();
        self.spi.transfer(commands).unwrap();
        self.chip_select_pin.set_high().unwrap();
    }

    fn send_data(&mut self, data: &mut [u8]) {
        let _ = self.data_command_pin.set_high();
        self.chip_select_pin.set_low().unwrap();
        self.spi.transfer(data).unwrap();
        self.chip_select_pin.set_high().unwrap();
    }

    fn set_frame_memory_from_raw(
        &mut self,
        image_buffer: &mut [u8],
        x: usize,
        mut y: usize,
        image_width: usize,
        mut image_height: usize,
    ) {
        y /= 8;
        image_height /= 8;

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

        self.send_command(&mut [0x21, x as u8, x_end as u8]);
        self.send_command(&mut [0x22, y as u8, y_end as u8]);
        for line in y..=y_end {
            self.send_data(&mut image_buffer[(line - y) * image_width / Self::PIXEL_PER_BYTE..][..image_width]);
        }
    }
}

impl<DCPin: OutputPin, CSPin: port::PinOps> Display<bool> for WideOledDisplay<DCPin, CSPin> {
    const PIXEL_PER_BYTE: usize = 8;
    const WIDTH: usize = 128;
    const HEIGHT: usize = 64;
    const LIGHT_COLOR: bool = true;
    const DARK_COLOR: bool = false;

    fn set_frame_memory_from_callback(
        &mut self,
        cb: impl Fn(usize, usize) -> bool,
        x: usize,
        mut y: usize,
        image_width: usize,
        mut image_height: usize,
    ) {
        y /= 8;
        image_height /= 8;

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

        self.send_command(&mut [0x21, x as u8, x_end as u8]);
        self.send_command(&mut [0x22, y as u8, y_end as u8]);
        for line in y..=y_end {
            for x in x..=x_end {
                let mut byte = 0;
                for y in (0..8).rev() {
                    byte <<= 1;
                    byte |= cb(x, line * 8 + y) as u8
                }
                self.send_data(&mut [byte]);
            }
        }
    }
}
