use core::fmt::Write;

use arduino_hal::pac::USART0;


pub struct Printer {
    pub serial: arduino_hal::Usart<
        USART0,
        arduino_hal::port::Pin<arduino_hal::port::mode::Input, arduino_hal::hal::port::PD0>,
        arduino_hal::port::Pin<arduino_hal::port::mode::Output, arduino_hal::hal::port::PD1>,
    >,
}

unsafe impl Sync for Printer {}

impl Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if !byte.is_ascii() {
                continue;
            }
            self.serial.write_byte(byte);
        }
        Ok(())
    }
}

pub static mut PRINTER: Option<Printer> = None;

#[allow(unused_macros)]
macro_rules! print {
    ($literal:expr, $($args:tt)*) => {
        write!(unsafe { crate::PRINTER.as_mut().unwrap() }, $literal, $($args)*).unwrap()
    };
    ($literal:expr) => {
        write!(unsafe { crate::PRINTER.as_mut().unwrap() }, $literal).unwrap()
    };
}

macro_rules! println {
    ($literal:expr, $($args:tt)*) => {
        #[allow(unused_unsafe)]
        writeln!(unsafe { crate::PRINTER.as_mut().unwrap() }, $literal, $($args)*).unwrap()
    };
    ($literal:expr) => {{
        #[allow(unused_unsafe)]
        writeln!(unsafe { crate::PRINTER.as_mut().unwrap() }, $literal).unwrap();
    }};
    () => {{
        #[allow(unused_unsafe)]
        writeln!(unsafe { crate::PRINTER.as_mut().unwrap() }).unwrap();
    }};
}

#[allow(unused_macros)]
macro_rules! dbg {
    ($v:expr) => {
        println!("[{}:{}] {} = {:?};", file!(), line!(), stringify!($v), $v);
    };
}

macro_rules! init_printer {
    ($dp:expr, $pins:expr, $baud_rate:expr) => {
        let serial = arduino_hal::default_serial!($dp, $pins, $baud_rate);
        unsafe {
            PRINTER = Some(Printer { serial });
        }
    };
}