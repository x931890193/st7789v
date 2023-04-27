use std::io::Write;
use std::{thread, time};
use embedded_graphics::drawable::Drawable;
use embedded_graphics::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, RgbColor};
use embedded_graphics::prelude::Primitive;
use st7789v::{ST7789V};
use embedded_hal::digital::v2::OutputPin;
use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use sysfs_gpio::{Direction, Pin};


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

    // display instance
    let mut display = ST7789V::with_cs(device, gpio.pin_cs, gpio.pin_dc, gpio.pin_rst).expect("Init display error!");
    let mut delay = Delay;
    display.size();
    display.init(&mut delay).expect("Init delay error!");
    display.address_window(0, 0, 320, 240).expect("address_window error");
    // draw image on WHITE background
    display.clear(Rgb565::WHITE).expect("clear: panic message");

    let image = ImageRawLE::new(include_bytes!("./assets/ferris.raw"), 86, 64);

    let image= &Image::new(&image, Point::zero());

    // image.draw(&mut display);
    display.draw_image(image).expect("draw_image error");
}
