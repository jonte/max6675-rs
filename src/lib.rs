//! A driver for MAX6675 using the embedded_hal SPI traits

#![cfg_attr(not(test), no_std)]

pub mod max6675 {
    use embedded_hal::blocking::spi::Transfer;
    use embedded_hal::digital::v2::OutputPin;

    pub struct Max6675<SPI, CS> {
        pub spi: SPI,
        pub cs: CS,
    }

    #[derive(Debug, PartialEq)]
    /// MAX6675-specific errors
    pub enum Error {
        BusError,
        ThermocoupleDisconnected,
    }

    /// Holds a "raw" reading - temperature as well as some diagnostic bits.
    struct Reading {
        temp: u16,
        is_open: bool,
        device_id: u8,
    }

    impl<SPI, CS, E> Max6675<SPI, CS>
    where
        SPI: Transfer<u8, Error = E>,
        CS: OutputPin,
    {
        pub fn new(spi: SPI, cs: CS) -> Self {
            Max6675 { spi, cs }
        }

        /// Return the temperature in degrees celcius
        pub fn get_temperature(&mut self) -> Result<f32, Error> {
            let reading = self.read_spi();
            match reading {
                Ok(reading) => {
                    if reading.is_open {
                        Err(Error::ThermocoupleDisconnected)
                    } else if reading.device_id != 0_u8 {
                        Err(Error::BusError)
                    } else {
                        Ok(reading.temp as f32 * 0.25)
                    }
                }
                Err(_) => Err(Error::BusError),
            }
        }

        /// Read a raw value from the MAX6675 over SPI
        fn read_spi(&mut self) -> Result<Reading, E> {
            let _ = self.cs.set_low();

            let mut buffer = [0u8; 2];

            self.spi.transfer(&mut buffer)?;

            let _ = self.cs.set_high();
            let raw_reading: u16 = (buffer[0] as u16) << 8 | buffer[1] as u16;

            Ok(Reading {
                temp: raw_reading >> 3,
                is_open: ((raw_reading & 0b00000000_00000100) >> 2) == 1,
                device_id: ((raw_reading & 0b00000000_00000010) >> 1) as u8,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct FakeSPI {
            raw_data: [u8; 2],
        }

        struct FakeCS;

        impl Transfer<u8> for FakeSPI {
            type Error = ();

            fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
                for (i, w) in data.iter_mut().enumerate() {
                    *w = self.raw_data[i];
                }
                Ok(data)
            }
        }

        impl OutputPin for FakeCS {
            type Error = ();
            fn set_low(&mut self) -> Result<(), <Self as OutputPin>::Error> {
                Ok(())
            }
            fn set_high(&mut self) -> Result<(), <Self as OutputPin>::Error> {
                Ok(())
            }
        }

        #[test]
        fn parse_temp() {
            assert_eq!(
                Max6675::new(
                    FakeSPI {
                        raw_data: [0b0111_1111, 0b1111_1000]
                    },
                    FakeCS
                )
                .get_temperature()
                .unwrap(),
                1023.75
            );

            assert_eq!(
                Max6675::new(
                    FakeSPI {
                        raw_data: [0b0000_0000, 0b0000_0000]
                    },
                    FakeCS
                )
                .get_temperature()
                .unwrap(),
                0.0
            );

            assert_eq!(
                Max6675::new(
                    FakeSPI {
                        raw_data: [0b0000_0000, 0b0000_0100]
                    },
                    FakeCS
                )
                .get_temperature()
                .unwrap_err(),
                Error::ThermocoupleDisconnected
            );

            assert_eq!(
                Max6675::new(
                    FakeSPI {
                        raw_data: [0b0000_0000, 0b0000_0010]
                    },
                    FakeCS
                )
                .get_temperature()
                .unwrap_err(),
                Error::BusError
            );
        }
    }
}
