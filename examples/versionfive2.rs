use std::io::Write;
use std::{thread, time};
use embedded_graphics::drawable::Drawable;
use embedded_graphics::{DrawTarget, text_style};
use embedded_graphics::fonts::{Font8x16, Text};
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw, ImageRawBE, ImageRawLE};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888, RgbColor};
use embedded_graphics::prelude::{Pixel, Primitive, Size};
use embedded_graphics::primitives::{Line, Circle};
use embedded_graphics::style::{PrimitiveStyle, TextStyle};
use st7789v::{ST7789V};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_blocking_spi_Transfer;
use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use sysfs_gpio::{Direction, Pin};
use st7789v::Rotate::{Rotate0, Rotate180, Rotate270, Rotate90};


// versionFive Gpio
pub const GPIOCHIP_BASE: u8 = 0;
pub const LCD_CS: u8 = GPIOCHIP_BASE + 49;
pub const LCD_RST: u8 = GPIOCHIP_BASE + 42;
pub const LCD_DC: u8 = GPIOCHIP_BASE + 44;
pub const LCD_BL: u8 = GPIOCHIP_BASE + 51;
// versionFive Gpio


struct MyPin(Pin);

impl OutputPin for MyPin {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(0).unwrap();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(1).unwrap();
        Ok(())
    }
}

struct Delay;

impl embedded_hal::blocking::delay::DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        let millis = time::Duration::from_millis(ms as u64);
        thread::sleep(millis);
    }
}

struct Gpio {
    pub pin_cs: MyPin,
    pub pin_rst: MyPin,
    pub pin_dc: MyPin,
    pub pin_bl: MyPin,
}

impl Gpio {
    fn new() -> Gpio{
        Gpio{
            pin_cs: MyPin(Pin::new(LCD_CS as u64)),
            pin_rst: MyPin(Pin::new(LCD_RST as u64)),
            pin_dc: MyPin(Pin::new(LCD_DC as u64)),
            pin_bl: MyPin(Pin::new(LCD_BL as u64)),
        }
    }
    pub fn init_gpio(&mut self) {
        self.pin_cs.0.export().expect("[init_dev] error ");
        self.pin_cs.0.set_direction(Direction::Out).expect("[init_dev] error ");

        self.pin_rst.0.export().expect("[init_dev] error ");
        self.pin_rst.0.set_direction(Direction::Out).expect("[init_dev] error ");

        self.pin_dc.0.export().expect("[init_dev] error ");
        self.pin_dc.0.set_direction(Direction::Out).expect("[init_dev] error ");

        self.pin_bl.0.export().expect("[init_dev] error ");
        self.pin_bl.0.set_direction(Direction::Out).expect("[init_dev] error ");

        self.pin_cs.0.set_value(1).expect("[init_dev] error ");
        self.pin_bl.0.set_value(1).expect("[init_dev] error ")
    }
}


pub struct HardwareSpi{
    pub spi: Spidev
}

// SPi Instance
impl HardwareSpi {
    // new HardwareSpi instance
    pub fn new(device_name: &str) -> Self {
        let mut spi = Spidev::open(device_name).expect(format!("open {} error", device_name).as_str());
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(10000000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options).expect(format!("spi configure {} error", device_name).as_str());
        HardwareSpi{
            spi
        }
    }
}

impl embedded_hal::blocking::spi::Write<u8> for HardwareSpi {
    type Error = ();

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(words).expect("1111");
        Ok(())
    }
}



fn main() {
    // for versionFive2
    let mut gpio = Gpio::new();
    gpio.init_gpio();
    // spi instance
    let device = HardwareSpi::new("/dev/spidev1.0");
    let width =  240;   // short side
    let height = 320;   // long side

    // display instance
    let mut display = ST7789V::with_cs(device, gpio.pin_cs, gpio.pin_dc, gpio.pin_rst, width, height).expect("Init display error!");
    let mut delay = Delay;
    display.init(&mut delay).expect("Init delay error!");
    display.set_rotate(Rotate270).expect("[set_rotate] error");

    display.address_window(0, 0, display.size().width as u16, display.size().height as u16).expect("address_window error");
    // draw image on WHITE background
    display.clear(Rgb565::WHITE).expect("clear: panic message");


    // Include the BMP file data.
    // let bmp_data = include_bytes!("./assets/111.bmp");

    // Parse the BMP file.
    // let bmp = Bmp::from_slice(bmp_data).unwrap().as_raw();

    // Draw the image with the top left corner at (10, 20) by wrapping it in
    // an embedded-graphics `Image`.

    // let image = ImageRawLE::new(&bmp.image_data(), 320, 240);
    // let image= &Image::new(&image, Point::new(0, 0));

    let image = ImageRawLE::new(include_bytes!("./assets/ferris.raw"), 86, 64);
    let image= &Image::new(&image, Point::new(50, 50));

    display.draw_image(&image).expect("[draw_image] error");

    let line =  Line::new(Point::new(0, 0), Point::new(width as i32, height as i32 )).into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 10));
    display.draw_line(&line).expect("[draw_line] error");

    let circle = Circle::new(Point::new(120, 160), 30).into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 10));
    display.draw_circle(&circle).expect("[draw_circle] error");

    let style = TextStyle::new(Font8x16, Rgb565::BLUE);

    let text = Text::new("hello world", Point::new(10, 100)).into_styled(style);
    display.draw_iter(text.into_iter());
    // release
    display.release().expect("[release display] error");
    // backlight
    gpio.pin_bl.0.unexport().expect("");
}
