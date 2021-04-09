//! examples/sdram.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy::logger;

use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::gpio::*;
use libdaisy::prelude::*;
use libdaisy::system;

use micromath::F32Ext;

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

        let sdram_size_bytes = libdaisy::sdram::Sdram::bytes();
        let sdram_size = sdram_size_bytes / core::mem::size_of::<u32>();

        info!(
            "SDRAM size: {} bytes, {} words starting at {:?}",
            sdram_size_bytes, sdram_size, &sdram[0] as *const _
        );

        // Make sure that we're not reading memory from a previous test run
        info!("Clear memory...");
        for item in sdram.iter_mut().take(sdram_size) {
            *item = 0.0;
        }

        info!("Write test pattern...");
        let mut data: f32 = 0.0;
        for item in sdram.iter_mut().take(sdram_size) {
            *item = data;
            data = (data + 1.0) % core::f32::MAX;
        }

        info!("Read test pattern...");
        let percent = (sdram_size as f64 / 100.0) as f32;
        data = 0.0;
        for (i, item) in sdram.iter_mut().enumerate().take(sdram_size) {
            assert!((*item - data).abs() < f32::EPSILON);
            data = (data + 1.0) % core::f32::MAX;

            if (i as f32 % (10.0 * percent)) == 0.0 {
                info!("{}% done", i as f32 / percent);
            }
        }
        info!("Test Success!");

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
