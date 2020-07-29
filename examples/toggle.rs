//! examples/toggle.rs
#![no_main]
#![no_std]
use rtic::cyccnt::U32Ext;

use log::info;

use libdaisy_rust::gpio::*;
use libdaisy_rust::hid;
use libdaisy_rust::prelude::*;
use libdaisy_rust::system;
use libdaisy_rust::MILICYCLES;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: SeedLed,
        switch1: hid::Switch<Daisy28<Input<PullUp>>>,
    }

    #[init( schedule = [interface_handler] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let mut system = system::System::init(ctx.core, ctx.device);

        let daisy28 = system
            .gpio
            .daisy28
            .take()
            .expect("Failed to get pin daisy28!")
            .into_pull_up_input();

        let switch1 = hid::Switch::new(daisy28);

        let now = ctx.start;
        ctx.schedule.interface_handler(now).unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
            switch1,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( schedule = [interface_handler], resources = [seed_led, switch1] )]
    fn interface_handler(ctx: interface_handler::Context) {
        static mut LED_IS_ON: bool = false;

        let switch1 = ctx.resources.switch1;
        switch1.update();

        if switch1.is_falling() {
            info!("Button pressed!");
            *LED_IS_ON = !(*LED_IS_ON);
            if *LED_IS_ON {
                ctx.resources.seed_led.set_high().unwrap();
            } else {
                ctx.resources.seed_led.set_low().unwrap();
            }
        }

        ctx.schedule
            .interface_handler(ctx.scheduled + MILICYCLES.cycles())
            .unwrap();
    }

    extern "C" {
        fn TIM4();
    }
};
