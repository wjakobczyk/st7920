#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

use hal::delay::Delay;
use hal::gpio::*;
use hal::rcc::RccExt;
use hal::spi::*;
use hal::stm32;
use stm32f4xx_hal as hal;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};

use st7920::ST7920;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let rcc = p.RCC.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(stm32f4xx_hal::time::MegaHertz(168))
            .freeze();

        let mut delay = Delay::new(cp.SYST, clocks);

        let gpiob = p.GPIOB.split();

        let sck = gpiob.pb3.into_alternate_af5();
        let mosi = gpiob.pb5.into_alternate_af5();
        let reset = gpiob.pb7.into_push_pull_output();
        let cs = gpiob.pb6.into_push_pull_output();

        let spi = Spi::spi1(
            p.SPI1,
            (sck, NoMiso, mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            stm32f4xx_hal::time::KiloHertz(600).into(),
            clocks,
        );

        let mut disp = ST7920::new(spi, reset, Some(cs), false);

        disp.init(&mut delay).expect("could not init display");
        disp.clear(&mut delay).expect("could not clear display");

        let c = Circle::new(Point::new(20, 20), 8)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
        let t = Text::new(
            "Hello Rust!",
            Point::new(40, 16),
            MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
        );

        c.draw(&mut disp).unwrap();
        t.draw(&mut disp).unwrap();

        disp.flush(&mut delay).expect("could not flush display");
    }

    loop {
        continue;
    }
}
