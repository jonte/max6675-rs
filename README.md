MAX6675 driver for use with `embedded_hal`
------------------------------------------

This driver parses the 12-bit temperature field reported by MAX6675 over SPI.
It is compatible with any device implementing the `embedded_hal` SPI traits.

The driver has been tested with the rp2040-hal crate on a Raspberry Pi Pico.
