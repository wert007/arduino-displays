use arduino_hal::{port, prelude::*, spi::{ChipSelectPin, self}};
use embedded_hal::digital::v2::OutputPin;

use super::Display;

const OLED_SPI_SETTINGS: spi::Settings = spi::Settings {
    data_order: spi::DataOrder::MostSignificantFirst,
    clock: spi::SerialClockRate::OscfOver2,
    mode: embedded_hal::spi::MODE_0,
};

// Something like this would be super practical to have a display, which only
// updates the parts, that changed of a display. This is faster and more
// efficent, but we cannot store this buffer, since it is too big.
// pub static mut BUFFER: [u8; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [0; DISPLAY_HEIGHT * DISPLAY_WIDTH];

pub struct OledDisplay<DCPin: OutputPin, CSPin: port::PinOps> {
    data_command_pin: DCPin,
    chip_select_pin: ChipSelectPin<CSPin>,
    spi: arduino_hal::Spi,
}

#[allow(dead_code)]
impl<DCPin: OutputPin, CSPin: port::PinOps> OledDisplay<DCPin, CSPin> {
    pub fn new(
        mut spi: arduino_hal::Spi,
        chip_select_pin: ChipSelectPin<CSPin>,
        mut reset_pin: impl OutputPin,
        data_command_pin: DCPin,
    ) -> Self {
        spi.reconfigure(OLED_SPI_SETTINGS).unwrap();

        let _ = reset_pin.set_high();
        arduino_hal::delay_ms(100);
        let _ = reset_pin.set_low();
        arduino_hal::delay_ms(100);
        let _ = reset_pin.set_high();
        arduino_hal::delay_ms(100);


        let mut result = OledDisplay {
            data_command_pin,
            spi,
            chip_select_pin,
        };

        result.send_command(&mut [
            0xae, //Set display off
            0xa0, //Set re-map
            0x51, 
            0xa1, //Set display start line
            0x00, 
            0xa2, //Set display offset
            0x00, 
            0xa4, //Normal Display
            0xa8, //Set multiplex ratio
            0x7f, 
            0xab, //Function Selection A
            0x01, //Enable internal VDD regulator
            0x81, //Set contrast
            0x80, 
            0xb1, //Set Phase Length
            0x31, 
            0xb3, //Set Front Clock Divider /Oscillator Frequency
            0xb1, 
            0xb5,  //represents GPIO pin output High
            0x03, 
            0xb6, //Set Second pre-charge Period
            0x0d, 
            0xbc, //Set Pre-charge voltage
            0x07, 
            0xbe, //Set VCOMH
            0x07, 
            0xd5, //Function Selection B
            0x02, //Enable second pre-charge
                     
            0xfd, 
            0x12, 
        
            0xaf, //Set display on

            0xa0,
            0x51,
        ]);

        arduino_hal::delay_ms(200);
        result.send_command(&mut [0xaf]);
        result
    }

    fn send_command(&mut self, command_and_args: &mut [u8]) {
        self.data_command_pin
            .set_low()
            .unwrap_or_else(|_| todo!("Implement good panic"));
        self.chip_select_pin.set_low().unwrap();
        self.spi.transfer(command_and_args).unwrap();
        self.chip_select_pin.set_high().unwrap();
    }

    fn send_data(&mut self, data: &mut [u8]) {
        self.data_command_pin
            .set_high()
            .unwrap_or_else(|_| todo!("Implement good panic"));
        self.chip_select_pin.set_low().unwrap();
        self.spi.transfer(data).unwrap();
        self.chip_select_pin.set_high().unwrap();
    }

    fn set_frame_memory_from_raw(
        &mut self,
        image_buffer: &mut [u8],
        mut x: usize,
        y: usize,
        mut image_width: usize,
        image_height: usize,
    ) {
        x /= 2;
        image_width /= 2;

        let x_end = if x + image_width >= Self::WIDTH / 2 {
            Self::WIDTH / 2 - 1
        } else {
            x + image_width - 1
        };
        let y_end = if y + image_height >= Self::HEIGHT {
            Self::HEIGHT - 1
        } else {
            y + image_height - 1
        };

        self.set_memory_area(x, y, x_end, y_end);
        self.send_data(image_buffer);

    }

    fn set_memory_area(&mut self, x: usize, y: usize, x_end: usize, y_end: usize) {
        self.send_command(&mut [0x15, x as u8, x_end as u8]);
        self.send_command(&mut [0x75, y as u8, y_end as u8]);
    }

    pub fn display_frame(&mut self) {
        // self.send_command(0x22);
        // self.send_data(&mut [0xC7]);
        // self.send_command(0x20);
        // self.block_until_idle();
    }
}

impl<DCPin: OutputPin, CSPin: port::PinOps> Display<u8> for OledDisplay<DCPin, CSPin> {
    fn set_frame_memory_from_callback(
        &mut self,
        f: impl Fn(usize, usize) -> u8,
        mut x: usize,
        y: usize,
        mut image_width: usize,
        image_height: usize,
    ) {
        x /= 2;
        image_width /= 2;

        if x > Self::WIDTH / 2 || y > Self::HEIGHT || image_width == 0 || image_height == 0 {
            return;
        }

        let x_end = if x + image_width >= Self::WIDTH / 2 {
            Self::WIDTH / 2 - 1
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
            for cursor in x..=x_end {
                let byte = f(cursor * 2, line) << 4 | (f(cursor * 2 + 1, line) & 0x0F);
                self.send_data(&mut [byte]);
            }
        }
    }



    const PIXEL_PER_BYTE: usize = 2;
    const HEIGHT: usize = 128;
    const WIDTH: usize = 128;
    const DARK_COLOR: u8 = 0x0;
    const LIGHT_COLOR: u8 = 0xF;
}
