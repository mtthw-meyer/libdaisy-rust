#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// #[macro_use(singleton)]
// extern crate cortex_m;

extern crate cfg_if;

use cortex_m::asm::delay as delay_cycles;

pub use stm32h7xx_hal::hal::digital::v2::InputPin;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::time::{Hertz, MegaHertz, U32Ext};

pub use cortex_m_log::println;

// extern crate panic_halt;

// use cortex_m_log::printer::Dummy;
// pub type Log = cortex_m_log::printer::dummy::Dummy;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        extern crate panic_semihosting;
        pub type Log = cortex_m_log::printer::semihosting::Semihosting<cortex_m_log::modes::InterruptFree, cortex_m_semihosting::hio::HStdout>;
    }
    else {
        extern crate panic_halt;
        use cortex_m_log::printer::Dummy;
        pub type Log = cortex_m_log::printer::dummy::Dummy;
    }
}

pub const MEGA: u32 = 1_000_000;
pub const MILLI: u32 = 1_000;
pub const AUDIO_FRAME_RATE_HZ: u32 = 1000;
pub const AUDIO_BLOCK_SIZE_HZ: usize = 48;
pub const AUDIO_SAMPLE_SIZE_HZ: usize = 48_001;
pub const CLOCK_RATE_MHZ: MegaHertz = MegaHertz(400_u32);

pub const CLK_CYCLES_PER_MS: u32 = CLOCK_RATE_MHZ.0 * MEGA / MILLI;

pub type FrameTimer = stm32h7xx_hal::timer::Timer<stm32h7xx_hal::stm32::TIM2>;

pub mod audio;
pub mod gpio;
pub mod hid;
pub mod system;

// Delay for ms, note if interrupts are active delay time will extend
pub fn delay_ms(ms: u32) {
    delay_cycles(ms * CLK_CYCLES_PER_MS);
}
