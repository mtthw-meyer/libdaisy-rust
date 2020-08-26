//! examples/sdram.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy_rust::logger;

use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy_rust::gpio::*;
use libdaisy_rust::prelude::*;
use libdaisy_rust::system;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: SeedLed,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);

        system.timer2.set_freq(500.ms());

        let sdram = system.sdram;

        let len = 8;
        for a in 0..len {
            info!("{}", sdram[a]);
        }

        for a in 0..len {
            sdram[a] = 2u32.pow(a as u32);
        }

        cortex_m::asm::dsb();
        for a in 0..len {
            info!("{}", sdram[a]);
            assert_eq!(2u32.pow(a as u32), sdram[a]);
        }

        init::LateResources {
            seed_led: system.gpio.led,
            timer2: system.timer2,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;

        ctx.resources.timer2.clear_irq();

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);
    }
};
