//! examples/button.rs
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use rtic::cyccnt::U32Ext;
use stm32h7xx_hal::time::Hertz;

use libdaisy_rust::gpio::{Input, PullUp};
use libdaisy_rust::*;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        button1: gpio::Daisy28<Input<PullUp>>,
    }

    #[init(schedule = [audio_callback])]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);
        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = ctx.start; // the start time of the system

        let button1 = system.gpio.daisy28.into_pull_up_input();

        // Schedule `blink` to run 1ms in the future
        ctx.schedule
            .audio_callback(now + (CLOCK_RATE_HZ.0).cycles())
            .unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
            button1,
        }
    }

    #[task( schedule = [audio_callback], resources = [seed_led, button1] )]
    fn audio_callback(ctx: audio_callback::Context) {
        if ctx.resources.button1.is_low().unwrap() {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }

        ctx.schedule
            .audio_callback(ctx.scheduled + (CLOCK_RATE_HZ.0).cycles())
            .unwrap();
    }

    // Declare unsused interrupt(s) for use by software tasks
    // https://docs.rs/stm32h7xx-hal/0.6.0/stm32h7xx_hal/enum.interrupt.html
    extern "C" {
        fn TIM2();
    }
};
