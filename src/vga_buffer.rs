use volatile::Volatile;
use core:: fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode(
            (background as u8) << 4 | (foreground as u8)
        ) // 4 bits for background, 4 bits for foreground
    }    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // guarantee that the struct's fields are laid out exactly like in C
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25; // 25 lines
const BUFFER_WIDTH: usize = 80; // 80 columns

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // static lifetime, which is the entire duration of the program
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // \n is a byte literal
            byte => {
                if self.column_position >= BUFFER_WIDTH { // if we're at the end of the line
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // we're writing to the last row
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar { // write the byte to the buffer
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1; // increment the column position
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read(); // read the character from the buffer
                self.buffer.chars[row - 1][col].write(character); // write the character to the previous row
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1); // clear the last row
        self.column_position = 0; // reset the column position
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ', // space character
            color_code: self.color_code,
        };
        
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank); // write the blank character to the row
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte), // 0x20 is the space character, 0x7e is the tilde
                // not part of printable ASCII range
                _ => self.write_byte(0xfe), // ■
            }
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
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, // mutable reference to the VGA buffer
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*))); // _print is a private function
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n")); // if there are no arguments, print a newline
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*))); // otherwise, print the arguments and a newline
}

#[doc(hidden)] // hide this function from the generated documentation
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap(); // write the arguments to the writer
}