#![no_std]

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

#[derive(Debug)]
pub enum Error<CommError, PinError> {
    Comm(CommError),
    Pin(PinError),
}

/// ST7735 instructions.
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

const WIDTH: i32 = 128;
const HEIGHT: i32 = 64;
const ROW_SIZE: usize = (WIDTH / 8) as usize;
const BUFFER_SIZE: usize = ROW_SIZE * HEIGHT as usize;
const X_ADDR_DIV: u8 = 16;

/// ST7735 driver to connect to TFT displays.
pub struct ST7920<SPI, RST, CS, DELAY>
where
    SPI: spi::Write<u8>,
    RST: OutputPin,
    CS: OutputPin,
{
    /// SPI
    spi: SPI,

    /// Reset pin.
    rst: RST,

    cs: Option<CS>,

    delay: DELAY,

    buffer: [u8; BUFFER_SIZE],
}

impl<SPI, RST, CS, DELAY, PinError, SPIError> ST7920<SPI, RST, CS, DELAY>
where
    SPI: spi::Write<u8, Error = SPIError>,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
    DELAY: DelayMs<u8> + DelayUs<u8>,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        rst: RST, //TODO option
        cs: Option<CS>,
        delay: DELAY,
    ) -> Self {
        let buffer = [0; BUFFER_SIZE];

        ST7920 {
            spi,
            rst,
            cs,
            delay,
            buffer,
        }
    }

    fn enable_cs(&mut self) -> Result<(), Error<SPIError, PinError>> {
        if let Some(cs) = self.cs.as_mut() {
            cs.set_high().map_err(Error::Pin)?;
            self.delay.delay_us(1);
        }
        Ok(())
    }

    fn disable_cs(&mut self) -> Result<(), Error<SPIError, PinError>> {
        if let Some(cs) = self.cs.as_mut() {
            self.delay.delay_us(1);
            cs.set_high().map_err(Error::Pin)?;
        }
        Ok(())
    }

    /// Runs commands to initialize the display.
    pub fn init(&mut self) -> Result<(), Error<SPIError, PinError>> {
        self.enable_cs()?;
        self.hard_reset()?;
        self.write_command(Instruction::BasicFunction)?;
        self.delay.delay_us(200);
        self.write_command(Instruction::DisplayOnCursorOff)?;
        self.delay.delay_us(100);
        self.write_command(Instruction::ClearScreen)?;
        self.delay.delay_ms(10);
        self.write_command(Instruction::EntryMode)?;
        self.delay.delay_us(100);
        self.write_command(Instruction::ExtendedFunction)?;
        self.delay.delay_ms(10);
        self.write_command(Instruction::GraphicsOn)?;
        self.delay.delay_ms(100);

        self.disable_cs()?;
        Ok(())
    }

    fn hard_reset(&mut self) -> Result<(), Error<(SPIError), PinError>> {
        self.rst.set_low().map_err(Error::Pin)?;
        self.delay.delay_ms(40);
        self.rst.set_high().map_err(Error::Pin)?;
        self.delay.delay_ms(40);
        Ok(())
    }

    fn write_command(&mut self, command: Instruction) -> Result<(), Error<SPIError, PinError>> {
        self.write_command_param(command, 0)
    }

    fn write_command_param(
        &mut self,
        command: Instruction,
        param: u8,
    ) -> Result<(), Error<SPIError, PinError>> {
        let command_param = command.to_u8().unwrap() | param;
        let cmd: u8 = 0xF8;

        self.spi
            .write(&[cmd, command_param & 0xF0, (command_param << 4) & 0xF0])
            .map_err(Error::Comm)?;

        Ok(())
    }

    fn write_data(&mut self, data: u8) -> Result<(), Error<SPIError, PinError>> {
        self.spi
            .write(&[0xFA, data & 0xF0, (data << 4) & 0xF0])
            .map_err(Error::Comm)?;
        Ok(())
    }

    fn set_address(&mut self, x: u8, y: u8) -> Result<(), Error<SPIError, PinError>> {
        const HALF_HEIGHT: u8 = HEIGHT as u8 / 2;

        self.write_command_param(
            Instruction::SetGraphicsAddress,
            if y < HALF_HEIGHT { y } else { y - HALF_HEIGHT },
        )?;
        self.write_command_param(
            Instruction::SetGraphicsAddress,
            if y < HALF_HEIGHT {
                x / X_ADDR_DIV
            } else {
                x / X_ADDR_DIV + (WIDTH as u8 / X_ADDR_DIV)
            },
        )?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error<SPIError, PinError>> {
        self.enable_cs()?;

        for y in 0..HEIGHT as u8 / 2 {
            self.set_address(0, y)?;

            for _x in 0..ROW_SIZE {
                self.write_data(0)?;
                self.write_data(0)?;
            }
        }

        self.disable_cs()?;
        Ok(())
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, val: u8) {
        let x_mask = 0x80 >> (x % 8);
        if val != 0 {
            self.buffer[y as usize * ROW_SIZE + x as usize / 8] |= x_mask;
        } else {
            self.buffer[y as usize * ROW_SIZE + x as usize / 8] &= !x_mask;
        }
    }

    pub fn flush(&mut self) -> Result<(), Error<SPIError, PinError>> {
        self.enable_cs()?;

        for y in 0..HEIGHT as u8 / 2 {
            self.set_address(0, y)?;

            let mut row_start = y as usize * ROW_SIZE;
            for x in 0..ROW_SIZE {
                self.write_data(self.buffer[row_start + x])?;
            }
            row_start += (HEIGHT as usize / 2) * ROW_SIZE;
            for x in 0..ROW_SIZE {
                self.write_data(self.buffer[row_start + x])?;
            }
        }

        self.disable_cs()?;
        Ok(())
    }

    pub fn flush_range(
        &mut self,
        x1: u8,
        y1: u8,
        mut w: u8,
        h: u8,
    ) -> Result<(), Error<SPIError, PinError>> {
        self.enable_cs()?;

        if w % 8 != 0 {
            w += 8; //make sure rightmost pixels are covered
        }

        if (w / 8) % 2 != 0 {
            w += 8; //need to send even number of bytes
        }

        let mut row_start = y1 as usize * ROW_SIZE;
        for y in y1..y1 + h {
            self.set_address(x1 / 8, y)?;

            for x in x1 / 8..(x1 + w) / 8 {
                self.write_data(self.buffer[row_start + x as usize])?;
                //TODO send in one SPI transaction
            }

            row_start += ROW_SIZE;
        }

        self.disable_cs()?;
        Ok(())
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::{
    drawable,
    pixelcolor::{
        raw::{RawData, RawU1},
        BinaryColor,
    },
    Drawing,
};

#[cfg(feature = "graphics")]
impl<SPI, CS, RST, DELAY, PinError, SPIError> Drawing<BinaryColor> for ST7920<SPI, CS, RST, DELAY>
where
    SPI: spi::Write<u8, Error = SPIError>,
    RST: OutputPin<Error = PinError>,
    CS: OutputPin<Error = PinError>,
    DELAY: DelayMs<u8> + DelayUs<u8>,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = drawable::Pixel<BinaryColor>>,
    {
        for drawable::Pixel(point, color) in item_pixels {
            self.set_pixel(
                point.x as u8,
                point.y as u8,
                RawU1::from(color).into_inner(),
            );
        }
    }
}
