//! examples/blinky2.rs
#![no_main]
#![no_std]

use rtic::cyccnt::U32Ext;

use panic_semihosting as _;

use cortex_m_log::printer::Printer;
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::stm32;

use libdaisy_rust::*;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        log: Log,
    }

    #[init( schedule = [blink] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let mut system = system::System::init(ctx.core, ctx.device);

        let now = ctx.start;
        ctx.schedule
            .blink(now + (CLK_CYCLES_PER_MS * 250).cycles())
            .unwrap();

        println!(
            system.log,
            "DMA1_STR0 Enabled: {} Pending: {}",
            stm32::NVIC::is_enabled(interrupt::DMA1_STR0),
            stm32::NVIC::is_pending(interrupt::DMA1_STR0),
        );

        init::LateResources {
            seed_led: system.gpio.led,
            log: system.log,
        }
    }

    #[idle]
    fn idle(ctx: idle::Context) -> ! {
        loop {}
    }

    #[task( schedule = [blink], resources = [seed_led, log] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);

        ctx.schedule
            .blink(ctx.scheduled + (CLK_CYCLES_PER_MS * 1000).cycles())
            .unwrap();
    }

    #[task( binds = DMA1_STR0, resources = [log] )]
    fn listener(ctx: listener::Context) {
        println!(ctx.resources.log, "Interrupting cow!")
    }

    #[task( binds = DMA1_STR1, resources = [log] )]
    fn listener2(ctx: listener2::Context) {
        static mut has_run: bool = false;

        if !(*has_run) {
            println!(ctx.resources.log, "Interrupting cow2!");
            *has_run = true;
        }
    }

    extern "C" {
        fn TIM4();
    }
};
