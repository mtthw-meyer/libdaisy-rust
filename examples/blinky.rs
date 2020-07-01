//! examples/blinky.rs
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use rtic::cyccnt::U32Ext;

use libdaisy_rust::*;
use stm32h7xx_hal::time::Hertz;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
    }

    #[init(schedule = [blink])]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);
        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = ctx.start; // the start time of the system

        // Schedule `blink` to run 250ms in the future
        ctx.schedule
            .blink(now + (CLOCK_RATE_HZ.0 / 4).cycles())
            .unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
        }
    }

    #[task( schedule = [blink], resources = [seed_led] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = false;

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);

        ctx.schedule
            .blink(ctx.scheduled + (CLOCK_RATE_HZ.0 / 4).cycles())
            .unwrap();
    }

    // Declare unsused interrupt(s) for use by software tasks
    // https://docs.rs/stm32h7xx-hal/0.6.0/stm32h7xx_hal/enum.interrupt.html
    extern "C" {
        fn TIM2();
    }
};
