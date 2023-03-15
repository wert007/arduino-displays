#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(int_log)]

use core::{
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{self, Ordering},
};

use arduino_hal::spi;

#[allow(unused_imports)]
use crate::{display::*, text::Writer};

#[macro_use]
mod print_macros;
use print_macros::*;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    const PRINT_PANIC: bool = false;
    if PRINT_PANIC {
        print!("Panicked");
        if let Some(message) = info.message() {
            print!(" with message \"");
            unsafe { PRINTER.as_mut().unwrap() }
                .write_fmt(*message)
                .unwrap();
            print!("\"");
        }
        if let Some(location) = info.location() {
            print!(" in {}", location.file());
        }
        println!();
    }
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

mod display;
mod text;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    init_printer!(dp, pins, 57600);
    println!("Start!");

    // Create SPI interface.
    let (spi, cs) = arduino_hal::Spi::new(
        dp.SPI,
        pins.d13.into_output(),
        pins.d11.into_output(),
        pins.d12.into_pull_up_input(),
        pins.d10.into_output(),
        spi::Settings::default(),
    );

    // let mut d = OledDisplay::new(
    //     spi,
    //     cs,
    //     pins.d8.into_output(),
    //     pins.d9.into_output(),
    //     // pins.d7.into_pull_up_input(),
    // );
    let mut d = EpaperDisplay::new(
        spi,
        cs,
        pins.d8.into_output(),
        pins.d9.into_output(),
        pins.d7.into_pull_up_input(),
    );

    println!("Inited!");
    let text = "Hello World!";
    let mut tick = 0;
    d.clear_frame_memory(d.dark_color());
    d.set_frame_memory_from_callback(|x, y| ((x + y) * 16 / 256) > 7, 0, 0, d.width(), d.height());
    d.write_string("Hewwo!", 0, 0, d.light_color(), d.dark_color());
    d.display_frame();
    println!("done");
    loop {
        d.write_string(&text[..tick], 0, 8, d.light_color(), d.dark_color());
        arduino_hal::delay_ms(500);
        tick += 1;
        if tick > text.len() {
            d.set_frame_memory_from_callback(|x, y| ((x + y) * 16 / 256) > 7, 0, 8, d.width(), 8);
            tick = 0;
        }
        d.display_frame();
    }
}
