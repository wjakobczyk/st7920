#![no_main]
#![no_std]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
use cortex_m_semihosting::hprintln;

use cortex_m_rt::entry;

use hal::delay::Delay;
use hal::stm32;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::gpio::*;
use stm32f4xx_hal::spi::*;

use stm32f4xx_hal::rcc::RccExt;

use cortex_m::peripheral::Peripherals;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Circle;

use st7920::ST7920;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        hprintln!("Hello, world!").unwrap();

        let rcc = p.RCC.constrain();

        // Configure clock to 168 MHz (i.e. the maximum) and freeze it
        let clocks = rcc
            .cfgr
            .sysclk(stm32f4xx_hal::time::MegaHertz(168))
            .freeze();

        // Get delay provider
        let delay = Delay::new(cp.SYST, clocks);

        let gpiod = p.GPIOD.split();
        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();

        let led = gpiod.pd12.into_push_pull_output();

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
            stm32f4xx_hal::time::KiloHertz(1200).into(),
            clocks,
        );

        let mut disp = ST7920::new(spi, reset, Some(cs), delay);

        disp.init().expect("could not init");
        //disp.test();

        disp.clear().expect("could not clear");

        disp.set_pixel(10, 10, 1);
        disp.set_pixel(11, 11, 1);
        disp.set_pixel(12, 12, 1);

        disp.set_pixel(110, 50, 1);
        disp.set_pixel(111, 50, 1);
        disp.set_pixel(112, 50, 1);
        disp.set_pixel(111, 50, 0);

        disp.draw(
            Circle::new(Point::new(30, 30), 15)
                .stroke(Some(BinaryColor::On))
                .into_iter(),
        );

        disp.flush_range(15, 15, 14, 31);

        loop {
            //delay.delay_ms(1_00_u16);
        }
    }

    loop {
        continue;
    }
}
