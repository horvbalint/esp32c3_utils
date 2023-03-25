use std::{thread, time::Duration};

use esp_idf_hal::gpio::{
    Level::{self, High, Low},
    Output, Pin, PinDriver,
};

use anyhow::Result;

const STEP_SEQ: [[Level; 4]; 8] = [
    [High, Low, Low, Low],
    [High, High, Low, Low],
    [Low, High, Low, Low],
    [Low, High, High, Low],
    [Low, Low, High, Low],
    [Low, Low, High, High],
    [Low, Low, Low, High],
    [High, Low, Low, High],
];
const STEPS_PER_REV: f32 = 512.;

pub enum Speed {
    Slow = 3,
    Mid = 2,
    Fast = 1,
}

pub struct Stepper<'a, In1: Pin, In2: Pin, In3: Pin, In4: Pin> {
    pub in1: PinDriver<'a, In1, Output>,
    pub in2: PinDriver<'a, In2, Output>,
    pub in3: PinDriver<'a, In3, Output>,
    pub in4: PinDriver<'a, In4, Output>,
    speed: u64,
}

impl<'a, In1: Pin, In2: Pin, In3: Pin, In4: Pin> Stepper<'a, In1, In2, In3, In4> {
    pub fn new(
        in1: PinDriver<'a, In1, Output>,
        in2: PinDriver<'a, In2, Output>,
        in3: PinDriver<'a, In3, Output>,
        in4: PinDriver<'a, In4, Output>,
    ) -> Self {
        Self {
            in1,
            in2,
            in3,
            in4,
            speed: Speed::Fast as u64,
        }
    }

    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed as u64;
    }

    pub fn rotate_angle_cw(&mut self, angle: u16) -> Result<()> {
        let step_count = (angle as f32 / 360_f32 * STEPS_PER_REV) as u32;
        self.step_n_cw(step_count)
    }

    pub fn rotate_angle_ccw(&mut self, angle: u16) -> Result<()> {
        let step_count = (angle as f32 / 360_f32 * STEPS_PER_REV) as u32;
        self.step_n_ccw(step_count)
    }

    pub fn step_n_cw(&mut self, step_count: u32) -> Result<()> {
        for _ in 0..step_count {
            self.step_one_cw()?
        }

        Ok(())
    }

    pub fn step_n_ccw(&mut self, step_count: u32) -> Result<()> {
        for _ in 0..step_count {
            self.step_one_ccw()?
        }

        Ok(())
    }

    pub fn step_one_cw(&mut self) -> Result<()> {
        for step in STEP_SEQ {
            self.in1.set_level(step[0])?;
            self.in2.set_level(step[1])?;
            self.in3.set_level(step[2])?;
            self.in4.set_level(step[3])?;

            thread::sleep(Duration::from_millis(self.speed));
        }

        Ok(())
    }

    pub fn step_one_ccw(&mut self) -> Result<()> {
        for step in STEP_SEQ {
            self.in1.set_level(step[3])?;
            self.in2.set_level(step[2])?;
            self.in3.set_level(step[1])?;
            self.in4.set_level(step[0])?;

            thread::sleep(Duration::from_millis(20));
        }

        Ok(())
    }

    pub fn rest(&mut self) -> Result<()> {
        self.in1.set_low()?;
        self.in2.set_low()?;
        self.in3.set_low()?;
        self.in4.set_low()?;

        Ok(())
    }
}
