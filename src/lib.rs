#![no_std]
#![allow(dead_code)]

// #[macro_use(singleton)]
// extern crate cortex_m;

use cortex_m::asm::delay as delay_cycles;

use stm32h7xx_hal::time::{Hertz, MegaHertz};

pub const MILLI: u32 = 1_000;
pub const AUDIO_FRAME_RATE_HZ: u32 = 1_000;
pub const AUDIO_BLOCK_SIZE: u16 = 48;
pub const AUDIO_SAMPLE_HZ: Hertz = Hertz(48_000);
pub const CLOCK_RATE_HZ: Hertz = Hertz(480_000_000_u32);

pub const MILICYCLES: u32 = CLOCK_RATE_HZ.0 / MILLI;

pub type FrameTimer = stm32h7xx_hal::timer::Timer<stm32h7xx_hal::stm32::TIM2>;

pub mod audio;
pub mod gpio;
pub mod hid;
pub mod logger;
pub mod prelude;
pub mod system;

// Delay for ms, note if interrupts are active delay time will extend
pub fn delay_ms(ms: u32) {
    delay_cycles(ms * MILICYCLES);
}

// pub fn ms_to_cycles(ms: u32) {

// }
