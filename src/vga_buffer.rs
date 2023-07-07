#![allow(dead_code)]

use core::fmt;

use spin::Mutex;
use volatile::Volatile; // 包装类型

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
    White = 15,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 确保其有着和 `C` 一样的内存对齐
struct ScreenChar {
    ascii_character : u8,
    color_code : ColorCode,
}


const BUFFER_HEIGHT : usize = 25;
const BUFFER_WIDTH : usize = 80;

#[repr(transparent)] // 确保类型和其单个成员有相同的内存布局
struct Buffer {
    chars : [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    colum_position : usize,
    color_code : ColorCode,
    buffer : &'static mut Buffer,
}

impl Writer {
    pub fn write_bytes(&mut self, byte : u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.colum_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.colum_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character : byte,
                    color_code,
                });
                self.colum_position += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.colum_position = 0;
    }
    pub fn write_string(&mut self, s : &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_bytes(byte),
                _ => self.write_bytes(0xfe),
            }
        }
    }
    pub fn clear_row(&mut self, row : usize) {
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
    fn write_str(&mut self, s : &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer {
        colum_position : 0,
        color_code : ColorCode::new(Color::Yellow, Color::Black),
        buffer : unsafe {
            &mut *(0xb8000 as *mut Buffer)
        },
    });
}

pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        colum_position : 0,
        color_code : ColorCode::new(Color::Yellow, Color::Black),
        buffer : unsafe {
            &mut *(0xb8000 as *mut Buffer)
        },
    };
    writer.write_bytes(b'H');
    writer.write_string("ello ");
    write!(writer, "The number are {} and {}", 42, 1.0/3.0).unwrap();
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
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
    
}






#[test_case]
fn test_println_simply() {
    println!("test_println_simply output");
}

#[test_case]
fn test_println_many() {
    for i in 0..200 {
        println!("test_println_simply output {}", i);
    }
}

#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;
    use core::fmt::Write;
    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(||{
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

}
