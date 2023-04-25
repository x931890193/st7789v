#![deny(unsafe_code, warnings)]
#![no_std]

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
    LeftToRight = 0b0000_0000,
    RightToLeft = 0b0100_0000,
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
    pub fn default() -> Self {
        MemAccCtrlConfig {
            color_order: ColorOrder::Rgb,
            latch_order: LatchOrder::LeftToRight,
            line_order: LineAddressOrder::TopToBottom,
            page_order: PageAddressOrder::TopToBottom,
            page_column_order: PageColumnOrder::NormalMode,
            column_order: ColumnAddressOrder::LeftToRight,
        }
    }

    pub fn color_order<'a>(&'a mut self, color_order: ColorOrder) -> &'a mut Self {
        self.color_order = color_order;
        self
    }

    pub fn latch_order<'a>(&'a mut self, latch_order: LatchOrder) -> &'a mut Self {
        self.latch_order = latch_order;
        self
    }

    pub fn line_order<'a>(&'a mut self, line_order: LineAddressOrder) -> &'a mut Self {
        self.line_order = line_order;
        self
    }

    pub fn page_order<'a>(&'a mut self, page_order: PageAddressOrder) -> &'a mut Self {
        self.page_order = page_order;
        self
    }

    pub fn page_column_order<'a>(&'a mut self, page_column_order: PageColumnOrder) -> &'a mut Self {
        self.page_column_order = page_column_order;
        self
    }

    pub fn column_order<'a>(&'a mut self, column_order: ColumnAddressOrder) -> &'a mut Self {
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
}

impl<SPI, CS, DC, RST, PinError, SpiError> ST7789V<SPI, CS, DC, RST, PinError, SpiError>
where
    SPI: spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
    DC: OutputPin<Error = PinError>,
    RST: OutputPin<Error = PinError>,
{
    /// Creates a new display instance
    pub fn new(spi: SPI, dc: DC, rst: RST) -> Self {
        ST7789V {
            spi,
            cfg: ST7789VConfig::new(dc, rst),
            _pin_err: PhantomData,
            _spi_err: PhantomData,
        }
    }

    /// Creates a new display instance with chip select pin
    pub fn with_cs(
        spi: SPI,
        mut cs: CS,
        dc: DC,
        rst: RST,
    ) -> Result<Self, Error<PinError, SpiError>> {
        cs.set_low().map_err(Error::Pin)?;

        let cfg = ST7789VConfig::with_cs(cs, dc, rst);
        Ok(ST7789V {
            spi,
            cfg,
            _pin_err: PhantomData,
            _spi_err: PhantomData,
        })
    }

    /// Creates a new display instance using a previously build display config
    pub fn with_config(
        spi: SPI,
        mut cfg: ST7789VConfig<CS, DC, RST>,
    ) -> Result<Self, Error<PinError, SpiError>> {
        if let Some(cs) = cfg.cs.as_mut() {
            cs.set_low().map_err(Error::Pin)?;
        }

        Ok(ST7789V {
            spi,
            cfg,
            _pin_err: PhantomData,
            _spi_err: PhantomData,
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
            .soft_reset(delay)?
            .sleep_out(delay)?
            .memory_access_control(MemAccCtrlConfig::default())?
            .color_mode(ColorFormat::RGB65K_CI8Bit, delay)?
            .porch_setting()?
            .gate_control()?
            .vcoms_setting()?
            .lcm_control()?
            .inversion_on()?
            .sleep_out(delay)?
            .display_on()?;

        Ok(())
    }

    /// This sets the RGB interface and control interface color format.
    pub fn color_mode<'a, DELAY>(
        &'a mut self,
        color_format: ColorFormat,
        delay: &mut DELAY,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    where
        DELAY: DelayMs<u16>,
    {
        self.command(Command::COLMOD, Some(&[color_format.value()]))?;
        delay.delay_ms(10);

        Ok(self)
    }

    /// This sets the porch setting.
    pub fn porch_setting<'a>(
        &'a mut self,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    {
        self.command(Command::PORCTRL, Some(&[0x0C, 0x0C, 0x00, 0x33, 0x33]))?;

        Ok(self)
    }

    /// This sets the gate control.
    pub fn gate_control<'a>(
        &'a mut self,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    {
        self.command(Command::GCTRL, Some(&[0x35]))?;

        Ok(self)
    }

    /// This sets the VCOMS setting.
    pub fn vcoms_setting<'a>(
        &'a mut self,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    {
        self.command(Command::VCOMS, Some(&[0x35]))?;

        Ok(self)
    }

    /// This sets the LCM control.
    pub fn lcm_control<'a>(
        &'a mut self,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    {
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
    pub fn sleep_in<'a, DELAY>(
        &'a mut self,
        delay: &mut DELAY,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    where
        DELAY: DelayMs<u16>,
    {
        self.command(Command::SLPIN, None)?;
        delay.delay_ms(5);

        Ok(self)
    }

    /// In this mode the DC/DC converter is enabled, internal display oscillator and the panel
    /// scanning is started.
    pub fn sleep_out<'a, DELAY>(
        &'a mut self,
        delay: &mut DELAY,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    where
        DELAY: DelayMs<u16>,
    {
        self.command(Command::SLPOUT, None)?;
        delay.delay_ms(120);

        Ok(self)
    }

    /// Leave normal mode and enter partial mode.
    pub fn partial_display_mode<'a>(
        &'a mut self,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::PTLON, None)?;

        Ok(self)
    }

    /// Leave partial mode and enter normal mode.
    pub fn normal_mode<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::NORON, None)?;

        Ok(self)
    }

    /// Display Inversion Off
    pub fn inversion_off<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::INVOFF, None)?;

        Ok(self)
    }

    /// Display Inversion On
    pub fn inversion_on<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::INVON, None)?;

        Ok(self)
    }

    /// The LCD enters DISPLAY OFF mode. In this mode, the output from frame memory is
    /// disabled and a blank page is inserted. This command does not change to the frame
    /// memory contents nor any other status. There will be no abnormal visible effect on the
    /// display.
    pub fn display_off<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::DISPOFF, None)?;

        Ok(self)
    }

    /// The LCD enters DISPLAY ON mode. The output from the frame memory is enabled. This
    /// command does not change the frame memory content nor any other status.
    pub fn display_on<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::DISPON, None)?;

        Ok(self)
    }

    /// Define read/write scanning direction of the frame memory.
    pub fn memory_access_control<'a>(
        &'a mut self,
        _config: MemAccCtrlConfig,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::MADCTL, Some(&[0x00]))?;

        Ok(self)
    }

    /// Idle mode off.
    pub fn idle_off<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::IDMOFF, None)?;

        Ok(self)
    }

    /// Idle mode on.
    pub fn idle_on<'a>(&'a mut self) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.command(Command::IDMON, None)?;

        Ok(self)
    }

    /// Sets the column address window.
    /// Each value represents one column line in the frame memory.
    ///
    /// `xs` must always be equal or less than `xe`. When `xs` or `xe` are greater than
    /// the maximum address, all data outside the range will be ignored.
    pub fn column_address<'a>(
        &'a mut self,
        xs: u16,
        xe: u16,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        if xs > xe {
            return Err(Error::InvalidColumnAddress);
        }

        self.command(
            Command::CASET,
            Some(&[
                (0x00) as u8,
                (xs + 0x22) as u8,
                (((xe + 0x22) - 1) >> 8) as u8,
                ((xe + 0x22) - 1) as u8,
                // (xs >> 8) as u8,
                // (xs & 0xFF) as u8,
                // (xe >> 8) as u8,
                // (xe & 0xFF) as u8,
            ]),
        )?;

        Ok(self)
    }

    /// Sets the row address window.
    /// Each value represents one page line in the frame memory.
    ///
    /// `rs` must always be equal or greater than `re`. Data outside the addressable
    /// space will be ignored.
    pub fn row_address<'a>(
        &'a mut self,
        rs: u16,
        re: u16,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        if rs > re {
            return Err(Error::InvalidRowAddress);
        }

        self.command(
            Command::RASET,
            Some(&[
                (0x00) as u8,
                (rs) as u8,
                ((re - 1) >> 8) as u8,
                (re - 1) as u8,
                // (rs >> 8) as u8,
                // (rs & 0xFF) as u8,
                // (re >> 8) as u8,
                // (re & 0xFF) as u8,
            ]),
        )?;

        Ok(self)
    }

    /// Sets the address window.
    pub fn address_window<'a>(
        &'a mut self,
        xs: u16,
        rs: u16,
        xe: u16,
        re: u16,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.column_address(xs, xe)?.row_address(rs, re)?;

        Ok(self)
    }

    /// Performs a hard reset. The display has to be initialized afterwards.
    pub fn hard_reset<'a, DELAY>(
        &'a mut self,
        delay: &mut DELAY,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    where
        DELAY: DelayMs<u16>,
    {
        self.cfg.rst.set_high().map_err(Error::Pin)?;
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
    pub fn soft_reset<'a, DELAY>(
        &'a mut self,
        delay: &mut DELAY,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>>
    where
        DELAY: DelayMs<u16>,
    {
        self.command(Command::SWRESET, None)?;
        delay.delay_ms(150);

        Ok(self)
    }

    /// Transfer data from MCU to the frame memory.
    pub fn mem_write<'a>(&'a mut self, data: &[u8]) -> Result<&'a Self, Error<PinError, SpiError>> {
        self.command(Command::RAMWR, Some(data))?;

        Ok(self)
    }

    /// Sets a single pixel to the given color
    pub fn pixel<'a>(
        &'a mut self,
        x: u16,
        y: u16,
        color: u16,
    ) -> Result<&'a Self, Error<PinError, SpiError>> {
        self.address_window(x, y, x, y)?;
        self.mem_write(&color.to_be_bytes())?;

        Ok(self)
    }

    pub fn pixels<'a>(
        &'a mut self,
        xs: u16,
        ys: u16,
        xe: u16,
        ye: u16,
        colors: &mut dyn Iterator<Item = u16>,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.address_window(xs, ys, xe, ye)?;
        self.mem_write(&[])?;

        for color in colors {
            self.data(&color.to_be_bytes())?;
        }

        Ok(self)
    }

    fn command<'a>(
        &'a mut self,
        cmd: Command,
        params: Option<&[u8]>,
    ) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.cfg.dc.set_low().map_err(Error::Pin)?;
        self.spi.write(&[cmd.value()]).map_err(Error::Spi)?;

        if let Some(params) = params {
            self.data(params)?;
        }

        Ok(self)
    }

    fn data<'a>(&'a mut self, data: &[u8]) -> Result<&'a mut Self, Error<PinError, SpiError>> {
        self.cfg.dc.set_high().map_err(Error::Pin)?;
        self.spi.write(data).map_err(Error::Spi)?;
        Ok(self)
    }
}
