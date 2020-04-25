use core::fmt;

use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
  Pink = 13,
  Yellow = 14,
  White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
  fn new(foreground: Color, background: Color) -> ColorCode {
    // Composes new u8 representing foreground and background
    // color by shifting background 4 bytes to the left and
    // then replacing the new 0s with the last 4 of the 
    // foreground. EX:
    // 0000xxxx << 4 = xxxx0000
    // xxxx0000 | 0000xxxx = xxxxxxxx
    ColorCode((background as u8) << 4 | (foreground as u8))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
  ascii_character: u8,
  color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer,
}

impl Writer {
  pub fn write_byte(&mut self, byte: u8) {
    match byte {
      // for the byte representing a new line, execute new_line function
      b'\n' => self.new_line(),
      // for any other byte
      byte => {
        // if the row is full, execute new_line function
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }
        // otherwise, write the bit to the screen
        
        // set the row and column to insert character
        // (bottom row and the column after the previously
        // written column
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        // Grab previously set color code from self
        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_character: byte,
          color_code,
        });
        self.column_position += 1;
      }
    }
  }

  // Write whole strings by converting them int bytes and then printing each
  // byte individually.
  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {
        // printable ASCII byte or newline
        0x20..=0x7e | b'\n' => self.write_byte(byte),
        // not part of printable ASCII range, print placeholder character
        _ => self.write_byte(0xfe),
      }
    }
  }

  // Shift all text up one row, removing top row if necessary,
  // reset column_position for new text inserts
  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let character = self.buffer.chars[row][col].read();
        self.buffer.chars[row-1][col].write(character);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_position = 0;
  }

  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(blank);
    }
  }
}

impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s);
    Ok(())
  }
}

lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
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

/// Prints the given formatted string to the
/// VGA text buffer through the global `WRITER`
/// instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  use core::fmt::Write;
  use x86_64::instructions::interrupts;

  interrupts::without_interrupts(|| {
      WRITER.lock().write_fmt(args).unwrap();
  });
}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_println_simple() {
  serial_print!("test_println... ");
  println!("test_println_simple output");
  serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
  serial_print!("test_println_many... ");
  for _ in 0..200 {
    println!("test_println_many output");
  }
  serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
  use core::fmt::Write;
  use x86_64::instructions::interrupts;
  
  serial_print!("test_println_output... ");

  let s = "Some test string that fits on a single line";
  interrupts::without_interrupts(|| {
    let mut writer = WRITER.lock();
    writeln!(writer, "\n{}", s).expect("writeln failed");
    for (i, c) in s.chars().enumerate() {
      let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
      assert_eq!(char::from(screen_char.ascii_character), c);
    }
  });

  serial_println!("[ok]");
}
