//! examples/button.rs
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m::asm::nop;
use rtic::cyccnt::U32Ext;

use log::info;

use debouncr::{debounce_4, Debouncer, Edge, Repeat4};

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
        button1_state: Debouncer<u8, Repeat4>,
    }

    #[init(schedule = [audio_callback])]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);

        let now = ctx.start;
        let button1 = system.gpio.daisy28.into_pull_up_input();

        ctx.schedule
            .audio_callback(now + (MILICYCLES).cycles())
            .unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
            button1,
            button1_state: debounce_4(),
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            nop();
        }
    }

    #[task( schedule = [audio_callback], resources = [seed_led, button1, button1_state] )]
    fn audio_callback(ctx: audio_callback::Context) {
        static mut LED_IS_ON: bool = false;

        // Poll button
        let pressed: bool = ctx.resources.button1.is_low().unwrap();

        // Update state
        let edge = ctx.resources.button1_state.update(pressed);

        // Handle event
        if edge == Some(Edge::Falling /*Edge::Rising*/) {
            info!("Button pressed!");
            *LED_IS_ON = !(*LED_IS_ON);
            if *LED_IS_ON {
                ctx.resources.seed_led.set_high().unwrap();
            } else {
                ctx.resources.seed_led.set_low().unwrap();
            }
        }

        ctx.schedule
            .audio_callback(ctx.scheduled + (MILICYCLES).cycles())
            .unwrap();
    }

    // Declare unsused interrupt(s) for use by software tasks
    // https://docs.rs/stm32h7xx-hal/0.6.0/stm32h7xx_hal/enum.interrupt.html
    extern "C" {
        fn TIM4();
    }
};
