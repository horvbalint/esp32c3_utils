use anyhow::Result;
use embedded_hal::spi::{Mode, Phase, Polarity};
use esp_idf_hal::{
    gpio::{AnyIOPin, IOPin, OutputPin},
    peripheral::Peripheral,
    spi::{
        config::{Config, Duplex},
        Dma, SpiAnyPins, SpiDeviceDriver, SpiDriver,
    },
};
use serde::{Deserialize, Serialize};

mod registers {
    #![allow(dead_code)]

    pub const SECONDS: u8 = 0x80u8.reverse_bits();
    pub const MINUTES: u8 = 0x82u8.reverse_bits();
    pub const HOURS: u8 = 0x84u8.reverse_bits();
    pub const DATE: u8 = 0x86u8.reverse_bits();
    pub const MONTH: u8 = 0x88u8.reverse_bits();
    pub const DAY: u8 = 0x8Au8.reverse_bits();
    pub const YEAR: u8 = 0x8Cu8.reverse_bits();
    pub const WRITE_PROTECT: u8 = 0x8Eu8.reverse_bits();
    pub const CLOCK_BURST: u8 = 0xBEu8.reverse_bits();
    pub const RAM: u8 = 0xC0u8.reverse_bits();
    pub const RAM_BURST: u8 = 0xFEu8.reverse_bits();
    pub const READ_FLAG: u8 = 0x1u8.reverse_bits();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar {
    pub year: u16,
    pub month: u8,
    pub date: u8,
    pub day: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clock {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub struct DS1302<'d> {
    spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
}

impl<'d> DS1302<'d> {
    pub fn new(
        spi_pin: impl Peripheral<P = impl SpiAnyPins> + 'd,
        cs_pin: impl Peripheral<P = impl OutputPin> + 'd,
        clk_pin: impl Peripheral<P = impl OutputPin> + 'd,
        data_pin: impl Peripheral<P = impl IOPin> + 'd,
    ) -> anyhow::Result<Self> {
        let spi = SpiDeviceDriver::new_single(
            spi_pin,
            clk_pin,
            data_pin,
            None as Option<AnyIOPin>,
            Dma::Disabled,
            Some(cs_pin),
            &Config {
                cs_active_high: true,
                data_mode: Mode {
                    polarity: Polarity::IdleLow,
                    phase: Phase::CaptureOnSecondTransition,
                },
                duplex: Duplex::Half3Wire,
                // bit_order: BitOrder::LsbFirst,
                ..Default::default()
            },
        )?;

        Ok(Self { spi })
    }

    pub fn init(&mut self) -> Result<()> {
        self.spi.write(&[registers::WRITE_PROTECT, 0x0])?; // disable write protect

        let seconds = self.get_seconds()?;
        if (seconds & registers::SECONDS) != 0 {
            println!("Writing seconds to 0");
            self.set_seconds(0x0)?; // disable chip halt flag (also resets the second counter)
        }

        Ok(())
    }

    fn get_seconds(&mut self) -> Result<u8> {
        let mut res = [0];
        self.spi
            .transfer(&mut res, &[registers::SECONDS | registers::READ_FLAG])?;

        Ok(res[0])
    }

    fn set_seconds(&mut self, seconds: u8) -> Result<()> {
        self.spi
            .write(&[registers::SECONDS, decimal_to_bcd(seconds).reverse_bits()])?;

        Ok(())
    }

    pub fn get_clock(&mut self) -> Result<Clock> {
        let mut res = [0; 3];
        self.spi
            .transfer(&mut res, &[registers::CLOCK_BURST | registers::READ_FLAG])?;

        Ok(Clock {
            hours: bcd_to_decimal(res[2]),
            minutes: bcd_to_decimal(res[1]),
            seconds: bcd_to_decimal(res[0]),
        })
    }

    pub fn get_calednar_and_clock(&mut self) -> Result<(Calendar, Clock)> {
        let mut res = [0; 8];
        self.spi
            .transfer(&mut res, &[registers::CLOCK_BURST | registers::READ_FLAG])?;

        Ok((
            Calendar {
                year: 2000 + bcd_to_decimal(res[6].reverse_bits()) as u16,
                month: bcd_to_decimal(res[4].reverse_bits()),
                date: bcd_to_decimal(res[3].reverse_bits()),
                day: bcd_to_decimal(res[5].reverse_bits()),
            },
            Clock {
                hours: bcd_to_decimal(res[2].reverse_bits()),
                minutes: bcd_to_decimal(res[1].reverse_bits()),
                seconds: bcd_to_decimal(res[0].reverse_bits()),
            },
        ))
    }

    pub fn set_calendar_and_date(&mut self, calendar: Calendar, clock: Clock) -> Result<()> {
        let bytes = [
            registers::CLOCK_BURST,
            decimal_to_bcd(clock.seconds).reverse_bits(),
            decimal_to_bcd(clock.minutes).reverse_bits(),
            decimal_to_bcd(clock.hours).reverse_bits(),
            decimal_to_bcd(calendar.date).reverse_bits(),
            decimal_to_bcd(calendar.month).reverse_bits(),
            decimal_to_bcd(calendar.day).reverse_bits(),
            decimal_to_bcd((calendar.year - 2000) as u8).reverse_bits(),
            0, // WP bit, if left out the year does not update for some reason
        ];

        self.spi.write(&bytes)?;

        Ok(())
    }
}

// Swap format from bcd to decmial
fn bcd_to_decimal(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F)
}

// Swap format from decimal to bcd
fn decimal_to_bcd(decimal: u8) -> u8 {
    ((decimal / 10) << 4) + (decimal % 10)
}
