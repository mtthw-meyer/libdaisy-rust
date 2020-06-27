#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use cortex_m::peripheral::DWT;

use rtic;
use stm32h7xx_hal::gpio;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

use crate::*;

#[allow(non_snake_case)]
pub struct System {
    pub log: Log,
    pub gpio: crate::gpio::GPIO,
    pub EXTI: stm32::EXTI,
    pub SYSCFG: stm32::SYSCFG,
    //pub audio: ?.
}

impl System {
    pub fn init(mut core: rtic::Peripherals, device: stm32::Peripherals) -> System {
        // Power
        let pwr = device.PWR.constrain();
        let vos = pwr.freeze();

        // Clocks
        let mut ccdr = device
            .RCC
            .constrain()
            .sys_ck(CLOCK_RATE_MHZ)
            .use_hse(16.mhz())
            .freeze(vos, &device.SYSCFG);

        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                // let mut log = cortex_m_log::printer::semihosting::InterruptOk::stdout().unwrap();
                let mut log = Dummy::new();
            }
            else {
                let mut log = Dummy::new();
            }
        }

        let mut timer2 = device
            .TIM2
            .timer(250.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        let gpio = crate::gpio::GPIO::init(gpioa, gpiob, gpioc, gpiod, gpioe, gpiof, gpiog);

        println!(log, "System init done!");

        System {
            log,
            gpio,
            EXTI: device.EXTI,
            SYSCFG: device.SYSCFG,
            // delay,
        }
    }
}
