#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate cfg_if;

extern crate stm32h7xx_hal;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::gpio::{Output, PushPull};
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

extern crate cortex_m_log;
pub use cortex_m_log::println;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        extern crate cortex_m_semihosting;
        extern crate panic_semihosting;
        pub type Log = cortex_m_log::printer::Semihosting<
            cortex_m_log::modes::InterruptOk,
            cortex_m_semihosting::hio::HStdout,
        >;
    }
    else {
        extern crate panic_halt;
        use cortex_m_log::printer::Dummy;
        pub type Log = cortex_m_log::printer::dummy::Dummy;
    }
}

pub const AUDIO_FRAME_RATE_HZ: u32 = 1000;
pub const AUDIO_BLOCK_SIZE_HZ: usize = 48;
pub const AUDIO_SAMPLE_SIZE_HZ: usize = 48_001;

pub type SeedLedPin = stm32h7xx_hal::gpio::gpioc::PC7<Output<PushPull>>;
pub type FrameTimer = stm32h7xx_hal::timer::Timer<stm32h7xx_hal::stm32::TIM2>;

pub mod gpio;

pub struct System {
    pub log: Log,
    pub led_pin: SeedLedPin,
    pub gpios: gpio::GPIO,
    pub timer1: Timer<TIM1>,
    pub timer2: Timer<TIM2>,
    pub timer3: Timer<TIM12>,
    pub timer4: Timer<TIM17>,
    // pub delay: stm32h7xx_hal::delay::Delay,
    //pub audio: ?.
}

impl System {
    pub fn init(
        core: stm32h7xx_hal::stm32::CorePeripherals,
        device: stm32h7xx_hal::stm32::Peripherals,
    ) -> System {
        // Power
        let pwr = device.PWR.constrain();
        let vos = pwr.freeze();

        // Clocks
        let mut ccdr = device
            .RCC
            .constrain()
            .sys_ck(400.mhz())
            .use_hse(16.mhz())
            .freeze(vos, &device.SYSCFG);

        let gpioa = device.GPIOA.split(&mut ccdr.ahb4);
        let gpiob = device.GPIOB.split(&mut ccdr.ahb4);
        let gpioc = device.GPIOC.split(&mut ccdr.ahb4);
        let gpiod = device.GPIOD.split(&mut ccdr.ahb4);
        let gpioe = device.GPIOE.split(&mut ccdr.ahb4);
        let gpiof = device.GPIOF.split(&mut ccdr.ahb4);
        let gpiog = device.GPIOG.split(&mut ccdr.ahb4);

        let led_pin = gpioc.pc7.into_push_pull_output();

        // let delay = cortex_device.SYST.delay(ccdr.clocks);

        // Timers
        let mut timer1 = device.TIM1.timer(125.ms(), &mut ccdr);
        timer1.listen(Event::TimeOut);

        let mut timer2 = device.TIM2.timer(250.ms(), &mut ccdr);
        timer2.listen(Event::TimeOut);

        let mut timer3 = device.TIM12.timer(500.ms(), &mut ccdr);
        timer3.listen(Event::TimeOut);

        let mut timer4 = device.TIM17.timer(1000.ms(), &mut ccdr);
        timer4.listen(Event::TimeOut);

        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                let mut log = cortex_m_log::printer::semihosting::InterruptOk::stdout().unwrap();
            }
            else {
                let mut log = Dummy::new();
            }
        }

        println!(log, "System init done!");

        System {
            log,
            led_pin,
            gpios: gpio::GPIO {},
            timer1,
            timer2,
            timer3,
            timer4,
            // delay,
        }
    }
}
