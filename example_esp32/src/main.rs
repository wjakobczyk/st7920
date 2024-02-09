// -*- coding: utf-8 -*-

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use enumset::EnumSet;
use esp_idf_hal::{self as hal, prelude::*};
use st7920::ST7920;

fn main() {
    let dp = Peripherals::take().unwrap();

    let config = hal::spi::config::Config::new()
        .baudrate(KiloHertz(1500).into())
        .data_mode(embedded_hal::spi::Mode {
            polarity: embedded_hal::spi::Polarity::IdleLow,
            phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
        });
    let sdi_dummy: Option<hal::gpio::Gpio1> = None;
    let cs_dummy: Option<hal::gpio::Gpio1> = None;
    let drvconfig = hal::spi::config::DriverConfig {
        dma: hal::spi::Dma::Disabled,
        intr_flags: EnumSet::EMPTY,
    };
    let spi_drv = hal::spi::SpiDriver::new(
        dp.spi2,
        dp.pins.gpio14,
        dp.pins.gpio13,
        sdi_dummy,
        &drvconfig,
    )
    .expect("could not init display SPI driver");
    let spi_dev_drv = hal::spi::SpiDeviceDriver::new(spi_drv, cs_dummy, &config)
        .expect("could not init display SPI device driver");

    let mut delay = hal::delay::Ets;
    let mut disp = ST7920::new(
        spi_dev_drv,
        hal::gpio::PinDriver::output(dp.pins.gpio33).unwrap(),
        Some(hal::gpio::PinDriver::output(dp.pins.gpio15).unwrap()),
        false,
    );
    disp.init(&mut delay).expect("could not init display");
    disp.clear(&mut delay).expect("could not clear display");

    let c =
        Circle::new(Point::new(20, 20), 8).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    let t = Text::new(
        "Hello Rust!",
        Point::new(40, 16),
        MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
    );

    c.draw(&mut disp).unwrap();
    t.draw(&mut disp).unwrap();

    disp.flush(&mut delay).expect("could not flush display");
}

// vim: ts=4 sw=4 expandtab
