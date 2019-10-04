# `ST7920`

This is a Rust driver library for LCD displays using the [ST7920] controller. It supports graphics mode of the controller, 128x64 in 1bpp. SPI connection to MCU is supported.

It implements [embedded-graphics] driver API.

It is platform independent as it uses [embedded-hal] APIs to access hardware.

The examples are based on the [stm32f4xx_hal] implementation of embedded-hal.



# Documentation

See [examples].

The controller supports 1 bit-per-pixel displays, so an off-screen buffer has to be used to provide random access to pixels.
Size of the buffer is 1024 bytes.

The buffer has to be flushed to update the display after a group of draw calls has been completed. The flush is not part of embedded-graphics API.

# License

This library is licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

[embedded-graphics]: https://docs.rs/embedded-graphics/0.6.0-alpha.2/embedded_graphics/
[embedded-hal]: https://docs.rs/embedded-hal/0.2.3/embedded_hal/
[stm32f4xx_hal]: https://docs.rs/stm32f4xx-hal/0.5.0/stm32f4xx_hal/
[examples]: https://github.com/wjakobczyk/st7920/tree/master/examples
[ST7920]: https://www.lcd-module.de/eng/pdf/zubehoer/st7920_chinese.pdf