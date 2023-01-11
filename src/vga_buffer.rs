// https://wiki.osdev.org/Text_UI for more info
// for now we will follow along with https://os.phil-opp.com/vga-text-mode/

use core::{
    fmt,
    ptr::{read_volatile, write_volatile},
};

use lazy_static::lazy_static;

use crate::mutex::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color: VgaColor::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VgaColor(u8);

impl VgaColor {
    pub fn new(text_color: Color, background_color: Color) -> Self {
        Self((background_color as u8) << 4 | (text_color as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: VgaColor,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    fn read(&self, row: usize, column: usize) -> ScreenChar {
        unsafe { read_volatile::<ScreenChar>(&self.chars[row][column] as *const ScreenChar) }
    }

    fn write(&mut self, row: usize, column: usize, char: ScreenChar) {
        unsafe {
            write_volatile(&mut self.chars[row][column] as *mut ScreenChar, char);
        }
    }
}

pub struct Writer {
    pub(crate) buffer: &'static mut Buffer,
    pub(crate) column_position: usize,

    pub(crate) color: VgaColor,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.write(
                    BUFFER_HEIGHT - 1,
                    self.column_position,
                    ScreenChar {
                        ascii_character: byte,
                        color_code: self.color,
                    },
                );

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // is ASCII char or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for column in 0..BUFFER_WIDTH {
                let char = self.buffer.read(row, column);
                self.buffer.write(row - 1, column, char);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.write(row, col, blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
