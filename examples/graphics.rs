#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

use hal::gpio::*;
use hal::rcc::RccExt;
use hal::spi::*;
use hal::timer::SysTimerExt;
use stm32f4xx_hal as hal;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;

use st7920::ST7920;

struct NoPin();

impl embedded_hal::digital::ErrorType for NoPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for NoPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (hal::pac::Peripherals::take(), Peripherals::take()) {
        let rcc = p.RCC.constrain();

        let clocks = rcc.cfgr.sysclk(hal::time::Hertz::MHz(168)).freeze();

        let mut delay = cp.SYST.delay(&clocks);

        let gpiob = p.GPIOB.split();

        let sck = gpiob.pb3.into_alternate();
        let mosi = gpiob.pb5.into_alternate();
        let reset = gpiob.pb7.into_push_pull_output();
        let cs = gpiob.pb6.into_push_pull_output();

        let spi = Spi1::new(
            p.SPI1,
            (sck, NoMiso::new(), mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            hal::time::Hertz::kHz(600),
            &clocks,
        );
        let spidev = ExclusiveDevice::new_no_delay(spi, NoPin());

        let mut disp = ST7920::new(spidev, reset, Some(cs), false);

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
