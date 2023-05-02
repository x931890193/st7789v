#![deny(unsafe_code, warnings)]
// #![no_std]

use core::marker::PhantomData;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

mod command;
use crate::command::Command;

#[cfg(feature = "graphics")]
mod graphics;

/// Errors
#[derive(Debug)]
pub enum Error<PinError, SpiError> {
    /// Invalid column address
    InvalidColumnAddress,
    /// Invalid row address
    InvalidRowAddress,
    /// Pin error
    Pin(PinError),
    /// SPI error
    Spi(SpiError),
}

/// RGB and control interface color format
#[allow(dead_code, non_camel_case_types)]
#[repr(u8)]
pub enum ColorFormat {
    /// RGB interface 65K, 8-bit data but for 16 Bit/Pixel
    RGB65K_CI8Bit = 0b0000_0101,
    /// RGB interface 65K, control interface 12 Bit/pixel
    RGB65K_CI12Bit = 0b0101_0011,
    /// RGB interface 65K, control interface 16 Bit/pixel
    RGB65K_CI16Bit = 0b0101_0101,
    /// RGB interface 65K, control interface 18 Bit/pixel
    RGB65K_CI18Bit = 0b0101_0110,
    /// RGB interface 65K, control interface 16M truncated
    RGB65K_CI16MTrunc = 0b0101_0111,
    /// RGB interface 262K, control interface 12 Bit/pixel
    RGB262K_CI12Bit = 0b0110_0011,
    /// RGB interface 262K, control interface 16 Bit/pixel
    RGB262K_CI16Bit = 0b0110_0101,
    /// RGB interface 262K, control interface 18 Bit/pixel
    RGB262K_CI18Bit = 0b0110_0110,
    /// RGB interface 262K, control interface 16M truncated
    RGB262K_CI16MTrunc = 0b0110_0111,
}

/// Rotate Rotate0 Rotate90 Rotate180 Rotate270
pub enum Rotate {
    Rotate0 = 0,
    Rotate90 = 90,
    Rotate180 = 180,
    Rotate270 = 270
}


impl ColorFormat {
    /// Get as COLMOD register value
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Page Address Order (MY)
pub enum PageAddressOrder {
    TopToBottom = 0b0000_0000,
    BottomToTop = 0b1000_0000,
}

impl PageAddressOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Column Address Order (MX)
pub enum ColumnAddressOrder {
    RightToLeft = 0b0000_0000,
    LeftToRight = 0b0100_0000,
}

impl ColumnAddressOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Page/Column Order (MV)
pub enum PageColumnOrder {
    NormalMode = 0b0000_0000,
    ReverseMode = 0b0010_0000,
}

impl PageColumnOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Line Address Order (ML)
pub enum LineAddressOrder {
    TopToBottom = 0b0000_0000,
    BottomToTop = 0b0001_0000,
}

impl LineAddressOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Color Order (RGB)
pub enum ColorOrder {
    Rgb = 0b0000_0000,
    Bgr = 0b0000_1000,
}

impl ColorOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Display Data Latch Order (MH)
pub enum LatchOrder {
    LeftToRight = 0b0000_0000,
    RightToLeft = 0b0000_0100,
}

impl LatchOrder {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Memory Access Control Config
pub struct MemAccCtrlConfig {
    color_order: ColorOrder,
    latch_order: LatchOrder,
    line_order: LineAddressOrder,
    page_order: PageAddressOrder,
    page_column_order: PageColumnOrder,
    column_order: ColumnAddressOrder,
}

impl MemAccCtrlConfig {

    // origin
    // 。。。。。。。。。。。。。。。   //
    // Left                 (0, 0)// RIGHT TOP
    //                            //
    //                            //
    //                            //
    // (320, 240)                 //
    //   2.0 inch LCD  Module     // BOTTOM

    // rotate 90
    // ############################
    // 。 (0, 0)              Module
    // 。                           D
    // 。                           C
    // 。                           L
    // 。                 (320, 240)2.0
    // ############################

    // rotate 180
    // ############################
    // 。                 (320, 240) Module
    // 。                           D
    // 。                           C
    // 。                           L
    // 。(0, 0)                     2.0
    // ############################


    // rotate 270
    // ############################
    // 。(320, 240)                 Module
    // 。                           D
    // 。                           C
    // 。                           L
    // 。                        (0, 0) 2.0
    // ############################


    pub fn default() -> Self {

        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::RightToLeft, // MIRROR_HORIZONTAL 水平镜像
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::BottomToTop, // Y
            page_column_order: PageColumnOrder::ReverseMode, // MIRROR_VERTICAL 垂直镜像
            column_order: ColumnAddressOrder::RightToLeft, // x
        }
    }

    pub fn rotate_0() -> Self {
        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::LeftToRight, // MIRROR_HORIZONTAL 水平镜像
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::TopToBottom, // Y
            page_column_order: PageColumnOrder::NormalMode, // MIRROR_VERTICAL 垂直镜像
            column_order: ColumnAddressOrder::RightToLeft, // x
        }
    }

    pub fn rotate_90() -> Self {
        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::LeftToRight, // MIRROR_HORIZONTAL 水平镜像
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::BottomToTop, // Y
            page_column_order: PageColumnOrder::NormalMode, // MIRROR_VERTICAL 垂直镜像
            column_order: ColumnAddressOrder::RightToLeft, // x
        }
    }

    pub fn rotate_180() -> Self {
        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::LeftToRight, // MIRROR_HORIZONTAL 水平镜像
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::BottomToTop, // Y
            page_column_order: PageColumnOrder::NormalMode, // MIRROR_VERTICAL 垂直镜像
            column_order: ColumnAddressOrder::LeftToRight, // x
        }
    }

    pub fn rotate_270() -> Self {
        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::LeftToRight, // MIRROR_HORIZONTAL 水平镜像
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::TopToBottom, // Y
            page_column_order: PageColumnOrder::NormalMode, // MIRROR_VERTICAL 垂直镜像
            column_order: ColumnAddressOrder::LeftToRight, // x
        }
    }

    pub fn color_order(&mut self, color_order: ColorOrder) -> &mut Self {
        self.color_order = color_order;
        self
    }

    pub fn latch_order(&mut self, latch_order: LatchOrder) -> &mut Self {
        self.latch_order = latch_order;
        self
    }

    pub fn line_order(&mut self, line_order: LineAddressOrder) -> &mut Self {
        self.line_order = line_order;
        self
    }

    pub fn page_order(&mut self, page_order: PageAddressOrder) -> &mut Self {
        self.page_order = page_order;
        self
    }

    pub fn page_column_order(&mut self, page_column_order: PageColumnOrder) -> &mut Self {
        self.page_column_order = page_column_order;
        self
    }

    pub fn column_order(&mut self, column_order: ColumnAddressOrder) -> &mut Self {
        self.column_order = column_order;
        self
    }

    pub fn value(self) -> u8 {
        self.color_order.value()
            | self.latch_order.value()
            | self.line_order.value()
            | self.page_order.value()
            | self.page_column_order.value()
            | self.column_order.value()
    }
}

/// ST7789V display driver config
pub struct ST7789VConfig<CS, DC, RST>
    where
        CS: OutputPin,
        DC: OutputPin,
        RST: OutputPin,
{
    /// Chip Select pin
    cs: Option<CS>,
    /// Data/Command pin
    dc: DC,
    /// Reset pin
    rst: RST,
}

impl<CS, DC, RST> ST7789VConfig<CS, DC, RST>
    where
        CS: OutputPin,
        DC: OutputPin,
        RST: OutputPin,
{
    /// Create a new display config
    pub fn new(dc: DC, rst: RST) -> Self {
        ST7789VConfig { cs: None, dc, rst }
    }

    /// Create a new display config with chip select pin
    pub fn with_cs(cs: CS, dc: DC, rst: RST) -> Self {
        ST7789VConfig {
            cs: Some(cs),
            dc,
            rst,
        }
    }

    /// Release the data/command and reset pin
    pub fn release(self) -> (DC, RST) {
        (self.dc, self.rst)
    }
}

/// ST7789V display driver
pub struct ST7789V<SPI, CS, DC, RST, PinError, SpiError>
    where
        SPI: spi::Write<u8>,
        CS: OutputPin,
        DC: OutputPin,
        RST: OutputPin,
{
    /// SPI
    spi: SPI,
    /// Config
    cfg: ST7789VConfig<CS, DC, RST>,

    _pin_err: PhantomData<PinError>,
    _spi_err: PhantomData<SpiError>,
    rotate: Rotate,
    width: u16,
    height: u16,
}

impl<SPI, CS, DC, RST, PinError, SpiError> ST7789V<SPI, CS, DC, RST, PinError, SpiError>
    where
        SPI: spi::Write<u8, Error=SpiError>,
        CS: OutputPin<Error=PinError>,
        DC: OutputPin<Error=PinError>,
        RST: OutputPin<Error=PinError>,
{
    /// Creates a new display instance
    pub fn new(spi: SPI, dc: DC, rst: RST, width: u16, height: u16) -> Self {
        ST7789V {
            spi,
            cfg: ST7789VConfig::new(dc, rst),
            _pin_err: PhantomData,
            _spi_err: PhantomData,
            rotate: Rotate::Rotate0,
            width,
            height
        }
    }

    /// Creates a new display instance with chip select pin
    pub fn with_cs(
        spi: SPI,
        mut cs: CS,
        dc: DC,
        rst: RST,
        width: u16,
        height: u16
    ) -> Result<Self, Error<PinError, SpiError>> {
        cs.set_low().map_err(Error::Pin)?;

        let cfg = ST7789VConfig::with_cs(cs, dc, rst);
        Ok(ST7789V {
            spi,
            cfg,
            _pin_err: PhantomData,
            _spi_err: PhantomData,
            rotate: Rotate::Rotate0,
            width,
            height,
        })
    }

    /// Creates a new display instance using a previously build display config
    pub fn with_config(
        spi: SPI,
        mut cfg: ST7789VConfig<CS, DC, RST>,
        width: u16,
        height: u16
    ) -> Result<Self, Error<PinError, SpiError>> {
        if let Some(cs) = cfg.cs.as_mut() {
            cs.set_low().map_err(Error::Pin)?;
        }

        Ok(ST7789V {
            spi,
            cfg,
            _pin_err: PhantomData,
            _spi_err: PhantomData,
            rotate: Rotate::Rotate0,
            width,
            height
        })
    }

    /// Release the SPI bus and display config. This will also raise the chip select pin.
    pub fn release(
        mut self,
    ) -> Result<(SPI, ST7789VConfig<CS, DC, RST>), Error<PinError, SpiError>> {
        if let Some(cs) = self.cfg.cs.as_mut() {
            cs.set_high().map_err(Error::Pin)?;
        }

        Ok((self.spi, self.cfg))
    }

    /// Initialize the display
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        self.hard_reset(delay)?
            .command(Command::MADCTL, Some(&[0x00]))?
            .command(Command::COLMOD, Some(&[0x05]))?
            .command(Command::INVON, None)?
            .command(Command::CASET, None)?
            .data(&[0x00])?
            .data(&[0x00])?
            .data(&[0x01])?
            .data(&[0x3f])?
            .command(Command::RASET, None)?
            .data(&[0x00])?
            .data(&[0x00])?
            .data(&[0x00])?
            .data(&[0x33])?
            .data(&[0x33])?
            .command(Command::GCTRL, Some(&[0x35]))?
            .command(Command::VCOMS, Some(&[0x1f]))?
            .command(Command::LCMCTRL,Some(&[0x2c]))?
            .command(Command::VDVVRHEN,Some(&[0x01]))?
            .command(Command::VRHS,Some(&[0x12]))?
            .command(Command::VDVS,Some(&[0x20]))?
            .command(Command::FRCTRL2,Some(&[0x0f]))?
            .command(Command::PWCTRL1,None)?
            .data(&[0xa4])?
            .data(&[0xa1])?
            .command(Command::E0,None)?
            .data(&[0xD0])?
            .data(&[0x08])?
            .data(&[0x11])?
            .data(&[0x08])?
            .data(&[0x0c])?
            .data(&[0x15])?
            .data(&[0x39])?
            .data(&[0x33])?
            .data(&[0x50])?
            .data(&[0x36])?
            .data(&[0x13])?
            .data(&[0x14])?
            .data(&[0x29])?
            .data(&[0x2d])?
            .command(Command::E1, None)?
            .data(&[0xd0])?
            .data(&[0x08])?
            .data(&[0x10])?
            .data(&[0x08])?
            .data(&[0x06])?
            .data(&[0x06])?
            .data(&[0x39])?
            .data(&[0x44])?
            .data(&[0x51])?
            .data(&[0x0b])?
            .data(&[0x16])?
            .data(&[0x14])?
            .data(&[0x2f])?
            .data(&[0x31])?
            .command(Command::INVON, None)?
            .command(Command::SLPOUT, None)?
            .command(Command::DISPON, None)?;
        Ok(())
    }

    pub fn set_rotate(&mut self, rotate: Rotate) -> Result<(), Error<PinError, SpiError>>{
        // let w = self.width;
        // let h = self.height;
        // TODO change x, y  or do there
        match rotate {
            Rotate::Rotate270 => {
                // self.memory_access_control(MemAccCtrlConfig::rotate_270())?;
            }
            Rotate::Rotate180 => {
                // self.memory_access_control(MemAccCtrlConfig::rotate_180())?;
            }
            Rotate::Rotate90 => {
                // self.memory_access_control(MemAccCtrlConfig::rotate_90())?;
            }
            _ => {
                // self.memory_access_control(MemAccCtrlConfig::rotate_0())?;
            }
        }
        self.rotate = rotate;

        Ok(())
    }


    /// This sets the RGB interface and control interface color format.
    pub fn color_mode<DELAY>(
        &mut self,
        color_format: ColorFormat,
        delay: &mut DELAY,
    ) -> Result<&mut Self, Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        self.command(Command::COLMOD, Some(&[color_format.value()]))?;
        delay.delay_ms(10);

        Ok(self)
    }

    /// This sets the porch setting.
    pub fn porch_setting(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::PORCTRL, Some(&[0x0C, 0x0C, 0x00, 0x33, 0x33]))?;

        Ok(self)
    }

    /// This sets the gate control.
    pub fn gate_control(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::GCTRL, Some(&[0x35]))?;

        Ok(self)
    }

    /// This sets the VCOMS setting.
    pub fn vcoms_setting(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::VCOMS, Some(&[0x35]))?;

        Ok(self)
    }

    /// This sets the LCM control.
    pub fn lcm_control(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::LCMCTRL, Some(&[0x2C]))?;

        self.command(Command::VDVVRHEN, Some(&[0x01]))?;
        self.command(Command::VRHS, Some(&[0x13]))?;
        self.command(Command::VDVS, Some(&[0x20]))?;
        self.command(Command::FRCTRL2, Some(&[0x0F]))?;
        self.command(Command::PWCTRL1, Some(&[0xA4, 0xA1]))?;
        self.command(Command::UNKNOWN_D6, Some(&[0xA1]))?;

        Ok(self)
    }

    /// This will put the LCD module into minimum power consumption mode.
    ///
    /// In this mode the DC/DC converter is stopped, the internal oscillator and the panel
    /// scanning is stopped. The MCU interface and memory are still working and the memory
    /// keeps its contents.
    pub fn sleep_in<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<&mut Self, Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        self.command(Command::SLPIN, None)?;
        delay.delay_ms(5);

        Ok(self)
    }

    /// In this mode the DC/DC converter is enabled, internal display oscillator and the panel
    /// scanning is started.
    pub fn sleep_out<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<&mut Self, Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        self.command(Command::SLPOUT, None)?;
        delay.delay_ms(120);

        Ok(self)
    }

    /// Leave normal mode and enter partial mode.
    pub fn partial_display_mode(
        &mut self,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::PTLON, None)?;

        Ok(self)
    }

    /// Leave partial mode and enter normal mode.
    pub fn normal_mode(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::NORON, None)?;

        Ok(self)
    }

    /// Display Inversion Off
    pub fn inversion_off(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::INVOFF, None)?;

        Ok(self)
    }

    /// Display Inversion On
    pub fn inversion_on(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::INVON, None)?;

        Ok(self)
    }

    /// The LCD enters DISPLAY OFF mode. In this mode, the output from frame memory is
    /// disabled and a blank page is inserted. This command does not change to the frame
    /// memory contents nor any other status. There will be no abnormal visible effect on the
    /// display.
    pub fn display_off(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::DISPOFF, None)?;

        Ok(self)
    }

    /// The LCD enters DISPLAY ON mode. The output from the frame memory is enabled. This
    /// command does not change the frame memory content nor any other status.
    pub fn display_on(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::DISPON, None)?;

        Ok(self)
    }

    /// Define read/write scanning direction of the frame memory.
    pub fn memory_access_control(
        &mut self,
        _config: MemAccCtrlConfig,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::MADCTL, Some(&[_config.value()]))?;

        Ok(self)
    }

    /// Idle mode off.
    pub fn idle_off(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::IDMOFF, None)?;

        Ok(self)
    }

    /// Idle mode on.
    pub fn idle_on(&mut self) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(Command::IDMON, None)?;

        Ok(self)
    }

    /// Sets the column address window.
    /// Each value represents one column line in the frame memory.
    ///
    /// `xs` must always be equal or less than `xe`. When `xs` or `xe` are greater than
    /// the maximum address, all data outside the range will be ignored.
    pub fn column_address(
        &mut self,
        xs: u16,
        xe: u16,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(
            Command::CASET,
            Some(&[
                (xs >> 8) as u8,
                (xs & 0xFF) as u8,
                (xe.wrapping_sub(1) >> 8) as u8,
                (xe.wrapping_sub(1) & 0xFF) as u8,
                // (0x00) as u8,
                // (xs & 0xFF) as u8,
                // (((xe + 0x22) - 1) >> 8) as u8,
                // (((xe + 0x22) - 1) & 0xFF) as u8,
            ]),
        )?;

        Ok(self)
    }

    /// Sets the row address window.
    /// Each value represents one page line in the frame memory.
    ///
    /// `rs` must always be equal or less than `re`. Data outside the addressable
    /// space will be ignored.
    pub fn row_address(
        &mut self,
        rs: u16,
        re: u16,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.command(
            Command::RASET,
            Some(&[
                (rs >> 8) as u8,
                (rs & 0xFF) as u8,
                (re.wrapping_sub(1) >> 8) as u8,
                (re.wrapping_sub(1) & 0xFF) as u8,
                // (0x00) as u8,
                // (rs & 0xFF) as u8,
                // ((re - 1) >> 8) as u8,
                // ((re - 1) & 0xFF) as u8,
            ]),
        )?;

        Ok(self)
    }

    /// Sets the address window.
    pub fn address_window(
        &mut self,
        xs: u16,
        rs: u16,
        xe: u16,
        re: u16,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        if xs > xe || rs > re {
            return Err(Error::InvalidColumnAddress);
        }
        self.column_address(xs, xe)?
            .row_address(rs, re)?
        .command(Command::RAMWR, None)?;
        Ok(self)
    }

    /// Performs a hard reset. The display has to be initialized afterwards.
    pub fn hard_reset<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<&mut Self, Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        if let Some(cs) = self.cfg.cs.as_mut() {
            cs.set_high().map_err(Error::Pin)?;
        }

        delay.delay_ms(1);
        self.cfg.rst.set_low().map_err(Error::Pin)?;
        delay.delay_ms(1);
        self.cfg.rst.set_high().map_err(Error::Pin)?;
        delay.delay_ms(120);

        Ok(self)
    }

    /// The display module performs a software reset.
    ///
    /// Registers are written with their SW reset default values. Frame memory contens are
    /// unaffected by this command.
    pub fn soft_reset<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<&mut Self, Error<PinError, SpiError>>
        where
            DELAY: DelayMs<u16>,
    {
        self.command(Command::SWRESET, None)?;
        delay.delay_ms(150);

        Ok(self)
    }

    fn transfer_x_y(&self, x: u16, y: u16) -> (u16, u16) {
        let mut start_x = x;
        let mut start_y = y;
        // change x, y
        match self.rotate {
            Rotate::Rotate90 => {
                start_x = self.width.wrapping_sub(x); // to avoid negative
                start_y = y
            }
            Rotate::Rotate180 => {
                start_x = x;
                start_y = self.height.wrapping_sub(y);  // to avoid negative
            }
            Rotate::Rotate270 => {
                start_x = self.width.wrapping_sub(x);  // to avoid negative
                start_y = self.height.wrapping_sub(y); // to avoid negative
            }
            _ => {}
        }
        (start_x, start_y)
    }

    /// Transfer data from MCU to the frame memory.
    pub fn mem_write(&mut self, data: &[u8]) -> Result<&Self, Error<PinError, SpiError>> {
        self.command(Command::RAMWR, Some(data))?;

        Ok(self)
    }

    /// Sets a single pixel to the given color
    pub fn pixel(
        &mut self,
        x: u16,
        y: u16,
        color: u16,
    ) -> Result<&Self, Error<PinError, SpiError>> {
        let (start_x, start_y) = self.transfer_x_y(x, y);
        self.address_window(start_x, start_y, start_x, start_y,)?; // for save bandwidth
        self.mem_write(&color.to_be_bytes())?;

        Ok(self)
    }

    pub fn pixels<'a>(
        &'a mut self,
        xs: u16,
        ys: u16,
        xe: u16,
        ye: u16,
        colors: &mut dyn Iterator<Item=u16>,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {

        let (start_x, start_y) = self.transfer_x_y(xs, ys);
        let (end_x, end_y) = self.transfer_x_y(xe, ye);

        let (min_x, max_x) = {
            if start_x > end_x {
                (end_x, start_x)
            } else {
                (start_x, end_x)
            }
        };

        let (min_y, max_y) = {
            if start_y > end_y {
                (end_y, start_y)
            } else {
                (start_y, end_y)
            }
        };

        self.address_window(min_x, min_y, max_x, max_y)?; // for save bandwidth
        self.mem_write(&[])?;
        if let Some(cs) = self.cfg.cs.as_mut() {
            cs.set_low().map_err(Error::Pin)?;
        }
        self.cfg.dc.set_high().map_err(Error::Pin)?;

        let colors_vec: Vec<u8> = colors.map(|x| x.to_be_bytes()).flatten().collect();

        let pixel_slice = colors_vec.as_slice();
        // fix this Cooperate with chatGPT
        // TODO: this is inconsistent in embedded-graphics between Rectangle and Image
        // See: https://github.com/jamwaffles/embedded-graphics/issues/182
        let reversed_chunks: Vec<&[u8]> = pixel_slice.chunks((self.width * self.width) as usize).rev().collect();
        let merged_data: &[u8] = &reversed_chunks.concat();

        for chunk in merged_data.chunks(4096) {
            self.data(&chunk)?;
        }
        if let Some(cs) = self.cfg.cs.as_mut() {
            cs.set_high().map_err(Error::Pin)?;
        }
        Ok(self)
    }

    fn command(
        &mut self,
        cmd: Command,
        params: Option<&[u8]>,
    ) -> Result<&mut Self, Error<PinError, SpiError>> {
        if let Some(cs) = self.cfg.cs.as_mut() {
            cs.set_low().map_err(Error::Pin)?;
        }
        self.cfg.dc.set_low().map_err(Error::Pin)?;
        self.spi.write(&[cmd.value()]).map_err(Error::Spi)?;

        if let Some(params) = params {
            if let Some(cs) = self.cfg.cs.as_mut() {
                cs.set_low().map_err(Error::Pin)?;
            }
            self.cfg.dc.set_high().map_err(Error::Pin)?;
            self.data(params)?;
            if let Some(cs) = self.cfg.cs.as_mut() {
                cs.set_high().map_err(Error::Pin)?;
            }
        }

        Ok(self)
    }

    fn data(&mut self, data: &[u8]) -> Result<&mut Self, Error<PinError, SpiError>> {
        self.spi.write(data).map_err(Error::Spi)?;
        Ok(self)
    }
}
