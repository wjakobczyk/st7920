//! # ST7920
//!
//! This is a Rust driver library for LCD displays using the [ST7920] controller.
//!
//! It supports graphics mode of the controller, 128x64 in 1bpp. SPI connection to MCU is supported.
//!
//! The controller supports 1 bit-per-pixel displays, so an off-screen buffer has to be used to provide random access to pixels.
//!
//! Size of the buffer is 1024 bytes.
//!
//! The buffer has to be flushed to update the display after a group of draw calls has been completed.
//! The flush is not part of embedded-graphics API.

#![no_std]
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

use embedded_hal::delay::DelayUs;
use embedded_hal::spi::{SpiDevice, SpiBusWrite};
use embedded_hal::digital::OutputPin;

#[derive(Debug)]
pub enum Error<CommError, PinError, DelayError> {
    Comm(CommError),
    Pin(PinError),
    Delay(DelayError),
}

/// ST7920 instructions.
#[derive(ToPrimitive)]
enum Instruction {
    BasicFunction = 0x30,
    ExtendedFunction = 0x34,
    ClearScreen = 0x01,
    EntryMode = 0x06,
    DisplayOnCursorOff = 0x0C,
    GraphicsOn = 0x36,
    SetGraphicsAddress = 0x80,
}

pub const WIDTH: u32 = 128;
pub const HEIGHT: u32 = 64;
const ROW_SIZE: usize = (WIDTH / 8) as usize;
const BUFFER_SIZE: usize = ROW_SIZE * HEIGHT as usize;
const X_ADDR_DIV: u8 = 16;

pub struct ST7920<SPI, RST, CS>
where
    SPI: SpiDevice,
    SPI::Bus: SpiBusWrite,
    RST: OutputPin,
    CS: OutputPin,
{
    /// SPI pin
    spi: SPI,

    /// Reset pin.
    rst: RST,

    /// CS pin
    cs: Option<CS>,

    buffer: [u8; BUFFER_SIZE],

    flip: bool,
}

impl<SPI, RST, CS, PinError, SPIError> ST7920<SPI, RST, CS>
where
    SPI: SpiDevice<Error = SPIError>,
    SPI::Bus: SpiBusWrite,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
{
    /// Create a new [`ST7920<SPI, RST, CS>`] driver instance that uses SPI connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use st7920::ST7920;
    ///
    /// let result = ST7920::new(spi, GPIO::new(pins.p01), None, false);
    /// assert_eq!(result, );
    /// ```
    pub fn new(spi: SPI, rst: RST, cs: Option<CS>, flip: bool) -> Self {
        let buffer = [0; BUFFER_SIZE];

        ST7920 {
            spi,
            rst,
            cs,
            buffer,
            flip,
        }
    }

    fn enable_cs<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().map_err(Error::Pin)?;
            delay.delay_us(1).map_err(Error::Delay)?;
        }
        Ok(())
    }

    fn disable_cs<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        if let Some(cs) = self.cs.as_mut() {
            delay.delay_us(1).map_err(Error::Delay)?;
            cs.set_high().map_err(Error::Pin)?;
        }
        Ok(())
    }

    /// Initialize the display controller
    pub fn init<DelayError: core::fmt::Debug, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.enable_cs(delay)?;
        self.hard_reset(delay)?;
        self.write_command(Instruction::BasicFunction, delay)?;
        delay.delay_us(200).map_err(Error::Delay)?;
        self.write_command(Instruction::DisplayOnCursorOff, delay)?;
        delay.delay_us(100).map_err(Error::Delay)?;
        self.write_command(Instruction::ClearScreen, delay)?;
        delay.delay_us(10 * 1000).map_err(Error::Delay)?;
        self.write_command(Instruction::EntryMode, delay)?;
        delay.delay_us(100).map_err(Error::Delay)?;
        self.write_command(Instruction::ExtendedFunction, delay)?;
        delay.delay_us(10 * 1000).map_err(Error::Delay)?;
        self.write_command(Instruction::GraphicsOn, delay)?;
        delay.delay_us(100 * 1000).map_err(Error::Delay)?;

        self.disable_cs(delay)?;
        Ok(())
    }

    fn hard_reset<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.rst.set_low().map_err(Error::Pin)?;
        delay.delay_us(40 * 1000).map_err(Error::Delay)?;
        self.rst.set_high().map_err(Error::Pin)?;
        delay.delay_us(40 * 1000).map_err(Error::Delay)?;
        Ok(())
    }

    fn write_command<DelayError: core::fmt::Debug, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        command: Instruction,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.write_command_param(command, 0, delay)
    }

    fn write_command_param<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        command: Instruction,
        param: u8,
        _delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        let command_param = command.to_u8().unwrap() | param;
        let cmd: u8 = 0xF8;

        self.spi
            .write(&[cmd, command_param & 0xF0, (command_param << 4) & 0xF0])
            .map_err(Error::Comm)?;

        Ok(())
    }

    fn write_data<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        data: u8,
        _delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.spi
            .write(&[0xFA, data & 0xF0, (data << 4) & 0xF0])
            .map_err(Error::Comm)?;
        Ok(())
    }

    fn set_address<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        x: u8,
        y: u8,
        delay: &mut Delay
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        const HALF_HEIGHT: u8 = HEIGHT as u8 / 2;

        self.write_command_param(
            Instruction::SetGraphicsAddress,
            if y < HALF_HEIGHT { y } else { y - HALF_HEIGHT },
            delay,
        )?;
        self.write_command_param(
            Instruction::SetGraphicsAddress,
            if y < HALF_HEIGHT {
                x / X_ADDR_DIV
            } else {
                x / X_ADDR_DIV + (WIDTH as u8 / X_ADDR_DIV)
            },
            delay,
        )?;

        Ok(())
    }

    /// Modify the raw buffer. 1 byte (u8) = 8 pixels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut st7920 = st7920::ST7920(...);
    /// // add crazy pattern
    /// st7920.modify_buffer(|x, y, v| {
    ///     if x % 2 == y % 2 {
    ///         v | 0b10101010
    ///     } else {
    ///         v
    ///     }
    /// });
    /// st7920.flush();
    /// ```
    pub fn modify_buffer(&mut self, f: fn(x: u8, y: u8, v: u8) -> u8) {
        for i in 0..BUFFER_SIZE {
            let row = i / ROW_SIZE;
            let column = i - (row * ROW_SIZE);
            self.buffer[i] = f(column as u8, row as u8, self.buffer[i]);
        }
    }

    /// clears the buffer but don't update the display
    pub fn clear_buffer(&mut self) {
        for i in 0..BUFFER_SIZE {
            self.buffer[i] = 0;
        }
    }

    /// Clear whole display area and clears the buffer
    pub fn clear<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.clear_buffer();
        self.flush(delay)?;
        Ok(())
    }

    /// Clear a buffer region.
    ///
    /// If the region is completely off screen,
    /// nothing will be done and Ok()) will be returned.
    /// If the given width or height are too big,
    /// width and height will be trimmed to the screen dimensions.
    pub fn clear_buffer_region<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        x: u8,
        mut y: u8,
        mut w: u8,
        mut h: u8,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        // Top-left is on screen and region has a width/height?
        if x < WIDTH as u8 && y < HEIGHT as u8 && w > 0 && h > 0 {
            // Limit width and height to right and bottom edge.
            if x.saturating_add(w) > WIDTH as u8 {
                w = WIDTH as u8 - x;
            }
            if y.saturating_add(h) > HEIGHT as u8 {
                h = HEIGHT as u8 - y;
            }

            self.enable_cs(delay)?;

            let mut adj_x = x;
            if self.flip {
                y = HEIGHT as u8 - (y + h);
                adj_x = WIDTH as u8 - (x + w);
            }

            let start = adj_x / 8;
            let mut right = adj_x + w;
            let end = (right / 8) + 1;

            let start_gap = adj_x % 8;

            right = end * 8;

            let end_gap = 8 - (right % 8);

            let mut row_start = y as usize * ROW_SIZE;
            for _ in y..y + h {
                for x in start..end {
                    let mut mask = 0xFF_u8;
                    if x == start {
                        mask = 0xFF_u8 >> start_gap;
                    }
                    if x == end {
                        mask &= 0xFF_u8 >> end_gap;
                    }

                    let pos = row_start + x as usize;
                    self.buffer[pos] &= !mask;
                }

                row_start += ROW_SIZE;
            }

            self.disable_cs(delay)?;
        }
        Ok(())
    }

    /// Draw pixel
    ///
    /// Doesn't draw anything, if the x or y coordinates are off canvas.
    ///
    /// Supported values are 0 and (not 0)
    #[inline]
    pub fn set_pixel(&mut self, x: u8, y: u8, val: u8) {
        if x < WIDTH as u8 && y < HEIGHT as u8 {
            self.set_pixel_unchecked(x, y, val);
        }
    }

    /// Draw pixel without canvas bounds checking.
    ///
    /// Supported values are 0 and (not 0)
    ///
    /// # Panics
    ///
    /// May panic or draw to undefined pixels, if x or y coordinates are off canvas.
    #[inline]
    pub fn set_pixel_unchecked(&mut self, mut x: u8, mut y: u8, val: u8) {
        if self.flip {
            y = (HEIGHT - 1) as u8 - y;
            x = (WIDTH - 1) as u8 - x;
        }
        let idx = y as usize * ROW_SIZE + x as usize / 8;
        let x_mask = 0x80 >> (x % 8);
        if val != 0 {
            self.buffer[idx] |= x_mask;
        } else {
            self.buffer[idx] &= !x_mask;
        }
    }

    /// Flush buffer to update entire display
    pub fn flush<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        delay: &mut Delay
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        self.enable_cs(delay)?;

        for y in 0..HEIGHT as u8 / 2 {
            self.set_address(0, y, delay)?;

            let mut row_start = y as usize * ROW_SIZE;
            for x in 0..ROW_SIZE {
                self.write_data(self.buffer[row_start + x], delay)?;
            }
            row_start += (HEIGHT as usize / 2) * ROW_SIZE;
            for x in 0..ROW_SIZE {
                self.write_data(self.buffer[row_start + x], delay)?;
            }
        }

        self.disable_cs(delay)?;
        Ok(())
    }

    /// Flush buffer to update region of the display
    ///
    /// If the region is completely off screen,
    /// nothing will be done and Ok()) will be returned.
    /// If the given width or height are too big,
    /// width and height will be trimmed to the screen dimensions.
    pub fn flush_region<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        x: u8,
        mut y: u8,
        mut w: u8,
        mut h: u8,
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        // Top-left is on screen and region has a width/height?
        if x < WIDTH as u8 && y < HEIGHT as u8 && w > 0 && h > 0 {
            // Limit width and height to right and bottom edge.
            if x.saturating_add(w) > WIDTH as u8 {
                w = WIDTH as u8 - x;
            }
            if y.saturating_add(h) > HEIGHT as u8 {
                h = HEIGHT as u8 - y;
            }

            self.enable_cs(delay)?;

            let mut adj_x = x;
            if self.flip {
                y = HEIGHT as u8 - (y + h);
                adj_x = WIDTH as u8 - (x + w);
            }

            let mut left = adj_x - adj_x % X_ADDR_DIV;
            let mut right = (adj_x + w) - 1;
            right -= right % X_ADDR_DIV;
            right += X_ADDR_DIV;

            if left > adj_x {
                left -= X_ADDR_DIV; //make sure rightmost pixels are covered
            }

            let mut row_start = y as usize * ROW_SIZE;
            self.set_address(adj_x, y, delay)?;
            for y in y..(y + h) {
                self.set_address(adj_x, y, delay)?;

                for x in left / 8..right / 8 {
                    self.write_data(self.buffer[row_start + x as usize], delay)?;
                }

                row_start += ROW_SIZE;
            }

            self.disable_cs(delay)?;
        }
        Ok(())
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics::{
    self, draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor, prelude::*,
};

#[cfg(feature = "graphics")]
impl<SPI, CS, RST, PinError, SPIError> OriginDimensions for ST7920<SPI, CS, RST>
where
    SPI: SpiDevice<Error = SPIError>,
    SPI::Bus: SpiBusWrite,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
{
    fn size(&self) -> Size {
        Size {
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

#[cfg(feature = "graphics")]
impl<SPI, CS, RST, PinError, SPIError> DrawTarget for ST7920<SPI, CS, RST>
where
    SPI: SpiDevice<Error = SPIError>,
    SPI::Bus: SpiBusWrite,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
{
    type Error = core::convert::Infallible;
    type Color = BinaryColor;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for p in pixels {
            let Pixel(coord, color) = p;

            #[cfg(not(feature = "graphics-unchecked"))]
            let in_bounds = coord.x >= 0 && coord.x < WIDTH as i32 &&
                            coord.y >= 0 && coord.y < HEIGHT as i32;
            #[cfg(feature = "graphics-unchecked")]
            let in_bounds = true;

            if in_bounds {
                let x = coord.x as u8;
                let y = coord.y as u8;
                let c = match color {
                    BinaryColor::Off => 0,
                    BinaryColor::On => 1,
                };
                self.set_pixel_unchecked(x, y, c);
            }
        }

        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<SPI, RST, CS, PinError, SPIError> ST7920<SPI, RST, CS>
where
    SPI: SpiDevice<Error = SPIError>,
    SPI::Bus: SpiBusWrite,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
{
    pub fn flush_region_graphics<DelayError, Delay: DelayUs<Error = DelayError>>(
        &mut self,
        region: (Point, Size),
        delay: &mut Delay,
    ) -> Result<(), Error<SPIError, PinError, DelayError>> {
        let mut width: u32 = region.1.width;
        let mut height: u32 = region.1.height;
        let mut x: i32 = region.0.x;
        let mut y: i32 = region.0.y;
        // Trim negative x position to zero. Reduce width accordingly.
        if x < 0 {
            width = width.saturating_sub((-x) as u32);
            x = 0;
        }
        // Trim negative y position to zero. Reduce height accordingly.
        if y < 0 {
            height = height.saturating_sub((-y) as u32);
            y = 0;
        }
        // Trim x, y, width and height to u8 range.
        x = x.min(u8::MAX as i32);
        y = y.min(u8::MAX as i32);
        width = width.min(u8::MAX as u32);
        height = height.min(u8::MAX as u32);
        self.flush_region(x as u8, y as u8, width as u8, height as u8, delay)
    }
}
