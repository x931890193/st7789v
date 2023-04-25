use embedded_graphics::drawable::Pixel;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::image::{Image, ImageDimensions, IntoPixelIter};
use embedded_graphics::pixelcolor::raw::{RawData, RawU16};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::{PrimitiveStyle, Styled};

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use crate::{Error, ST7789V};

impl<SPI, CS, DC, RST, PinError, SpiError> ST7789V<SPI, CS, DC, RST, PinError, SpiError>
where
    SPI: spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
    DC: OutputPin<Error = PinError>,
    RST: OutputPin<Error = PinError>,
{
    fn fill_rect(
        &mut self,
        item: &dyn Dimensions,
        colors: &mut dyn Iterator<Item = u16>,
    ) -> Result<(), Error<PinError, SpiError>> {
        let sx = item.top_left().x as u16;
        let sy = item.top_left().y as u16;
        let ex = item.bottom_right().x as u16;
        let ey = item.bottom_right().y as u16;

        self.pixels(sx, sy, ex, ey, colors)?;

        Ok(())
    }
}

impl<SPI, CS, DC, RST, PinError, SpiError> DrawTarget<Rgb565>
    for ST7789V<SPI, CS, DC, RST, PinError, SpiError>
where
    SPI: spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
    DC: OutputPin<Error = PinError>,
    RST: OutputPin<Error = PinError>,
{
    type Error = Error<PinError, SpiError>;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let color = RawU16::from(pixel.1).into_inner();
        let x = pixel.0.x as u16;
        let y = pixel.0.y as u16;

        self.pixel(x, y, color)?;

        Ok(())
    }

    fn draw_rectangle(
        &mut self,
        item: &Styled<Rectangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        // filled rect can be rendered into frame window directly
        if item.style.fill_color.is_some() {
            let mut colors = item.into_iter().map(|p| RawU16::from(p.1).into_inner());

            self.fill_rect(item, &mut colors)
        } else if let Some(_color) = item.style.stroke_color {
            if item.style.stroke_width == 0 {
                return Ok(()); // nothing to draw
            }
            // let sw = item.style.stroke_width as u16;

            // TODO: construct rectangle as 4 frames
            self.draw_iter(item)
        } else {
            // if we don't know what this rect is, draw individual pixels
            self.draw_iter(item)
        }
    }

    fn draw_image<'a, 'b, I>(&mut self, item: &'a Image<'b, I, Rgb565>) -> Result<(), Self::Error>
    where
        &'b I: IntoPixelIter<Rgb565>,
        I: ImageDimensions,
    {
        // TODO: this is inconsistent in embedded-graphics between Rectangle and Image
        // See: https://github.com/jamwaffles/embedded-graphics/issues/182
        let sx = item.top_left().x as u16;
        let sy = item.top_left().y as u16;
        let ex = (item.bottom_right().x - 1) as u16;
        let ey = (item.bottom_right().y - 1) as u16;
        let mut colors = item.into_iter().map(|p| RawU16::from(p.1).into_inner());

        self.pixels(sx, sy, ex, ey, &mut colors)?;

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(172, 320)
    }
}
