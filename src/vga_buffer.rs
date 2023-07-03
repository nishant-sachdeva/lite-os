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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
                self.buffer.chars[row][col] = ScreenChar { // write the byte to the buffer
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1; // increment the column position
            }
        }
    }

    fn new_line(&mut self) {}

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

pub fn print_something() {
    let mut vga_writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black), // yellow text on black background
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, // mutable reference to the VGA buffer
    };
    
    vga_writer.write_byte(b'H');
    vga_writer.write_string("ello! ");
    vga_writer.write_string("Wörld!");
    vga_writer.write_string("This is a test of the VGA buffer.");
}