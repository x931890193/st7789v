/// LCD Command
#[allow(unused, non_camel_case_types)]
#[repr(u8)]
pub enum Command {
    /// No operation
    NOP = 0x00,
    /// Software reset
    SWRESET = 0x01,
    /// Read display ID
    RDDID = 0x04,
    /// Read display status
    RDDST = 0x09,
    /// Read display power
    RDDPM = 0x0A,
    /// Read display MAD?
    RDD_MADCTL = 0x0B,
    /// Read display pixel
    RDD_COLMOD = 0x0C,
    /// Read display image
    RDDIM = 0x0D,
    /// Read display signal
    RDDSM = 0x0E,
    /// Read display self-diagnostic result
    RDDSDR = 0x0F,
    /// Sleep in
    SLPIN = 0x10,
    /// Sleep out
    SLPOUT = 0x11,
    /// Partial mode on
    PTLON = 0x12,
    /// Partial off (Normal)
    NORON = 0x13,
    /// Display inversion off
    INVOFF = 0x20,
    /// Display inversion on
    INVON = 0x21,
    /// Display inversion on (gamma?)
    GAMSET = 0x26,
    /// Display off
    DISPOFF = 0x28,
    /// Display on
    DISPON = 0x29,
    /// Column address set
    CASET = 0x2A,
    /// Row address set
    RASET = 0x2B,
    /// Memory write
    RAMWR = 0x2C,
    /// Memory read
    RAMRD = 0x2E,
    /// Partial start/end address set
    PTLAR = 0x30,
    /// Vertical scrolling definition
    VSCRDEF = 0x33,
    /// Tearing effect line off
    TEOFF = 0x34,
    /// Tearing effect line on
    TEON = 0x35,
    /// Memory data access control
    MADCTL = 0x36,
    /// Verical scrolling start address
    VSCRSADD = 0x37,
    /// Idle mode off
    IDMOFF = 0x38,
    /// Idle mode on
    IDMON = 0x39,
    /// Interface pixel format
    COLMOD = 0x3A,
    /// Memory write continue
    RAMWRC = 0x3C,
    /// Memory read continue
    RAMRDC = 0x3E,
    /// Set tear scanline
    TESCAN = 0x44,
    /// Get scanline
    RDTESCAN = 0x45,
    /// Write display brightness
    WRDISBV = 0x51,
    /// Read display brightness value
    RDDISBV = 0x52,
    /// Write CTRL display
    WRCTRLD = 0x53,
    /// Read CTRL value display
    RDCTRLD = 0x54,
    /// Write content adaptive brightness control and color enhancement
    WRCACE = 0x55,
    /// Read content adaptive brightness control
    RDCABC = 0x56,
    /// Write CABC minimum brightness
    WRCABCMB = 0x5E,
    /// Read CABC minimum brightness
    RDCABCMB = 0x5F,
    /// Read Automatic brightness control self-diagnostic result
    RDABCSDR = 0x68,
    /// Porch setting
    PORCTRL = 0xB2,
    /// Gate control
    GCTRL = 0xB7,
    /// VCOMS setting
    VCOMS = 0xBB,
    /// LCM control
    LCMCTRL = 0xC0,
    /// VCV and VRH command enable
    VDVVRHEN = 0xC2,
    /// VRH Set
    VRHS = 0xC3,
    /// VDVS Set
    VDVS = 0xC4,
    /// Frame Rate Control in Normal Mode
    FRCTRL2 = 0xC6,
    /// Power control 1
    PWCTRL1 = 0xD0,
    /// Unknown D6
    UNKNOWN_D6 = 0xD6,

    /// Read ID1
    RDID1 = 0xDA,
    /// Read ID2
    RDID2 = 0xDB,
    /// Read ID3
    RDID3 = 0xDC,

    E0 = 0xE0,
    E1 = 0xE1
}

impl Command {
    /// Get command as value.
    pub fn value(self) -> u8 {
        self as u8
    }
}
