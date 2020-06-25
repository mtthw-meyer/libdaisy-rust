#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]


use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::gpio;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

use crate::*;

#[allow(non_snake_case)]
pub struct System {
    pub log: Log,
    pub timer1: Timer<TIM1>,
    pub timer2: Timer<TIM2>,
    pub timer3: Timer<TIM12>,
    pub timer4: Timer<TIM17>,
    pub gpio: crate::gpio::GPIO,
    pub EXTI: stm32::EXTI,
    pub SYSCFG: stm32::SYSCFG,
    // pub delay: stm32h7xx_hal::delay::Delay,
    //pub audio: ?.
}

impl System {
    pub fn init(
        core: stm32::CorePeripherals,
        device: stm32::Peripherals,
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

        // let delay = cortex_device.SYST.delay(ccdr.clocks);

        // Timers
        let mut timer1 = device.TIM1.timer(125.ms(), ccdr.peripheral.TIM1, &mut ccdr.clocks);
        timer1.listen(Event::TimeOut);

        let mut timer2 = device.TIM2.timer(250.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        let mut timer3 = device.TIM12.timer(500.ms(), ccdr.peripheral.TIM12, &mut ccdr.clocks);
        timer3.listen(Event::TimeOut);

        let mut timer4 = device.TIM17.timer(1000.ms(), ccdr.peripheral.TIM17, &mut ccdr.clocks);
        timer4.listen(Event::TimeOut);

        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                // let mut log = cortex_m_log::printer::semihosting::InterruptOk::stdout().unwrap();
                let mut log = Dummy::new();
            }
            else {
                let mut log = Dummy::new();
            }
        }

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let gpio = crate::gpio::GPIO::init(
            gpioa,
            gpiob,
            gpioc,
            gpiod,
            gpioe,
            gpiof,
            gpiog,
        );

        println!(log, "System init done!");

        System {
            log,
            timer1,
            timer2,
            timer3,
            timer4,
            gpio,
            EXTI: device.EXTI,
            SYSCFG: device.SYSCFG,
            // delay,
        }
    }
}

