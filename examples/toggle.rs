//! examples/toggle.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy::logger;

use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::gpio::*;
use libdaisy::hid;
use libdaisy::prelude::*;
use libdaisy::system;
use stm32h7xx_hal::time::Hertz;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: SeedLed,
        switch1: hid::Switch<Daisy28<Input<PullUp>>>,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);

        let daisy28 = system
            .gpio
            .daisy28
            .take()
            .expect("Failed to get pin daisy28!")
            .into_pull_up_input();

        system.timer2.set_freq(Hertz(100));

        let switch1 = hid::Switch::new(daisy28, hid::SwitchType::PullUp);

        init::LateResources {
            seed_led: system.gpio.led,
            switch1,
            timer2: system.timer2,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led, switch1] )]
    fn interface_handler(ctx: interface_handler::Context) {
        static mut LED_IS_ON: bool = false;

        ctx.resources.timer2.clear_irq();
        let switch1 = ctx.resources.switch1;
        switch1.update();

        if switch1.is_falling() {
            info!("Button pressed!");
            if *LED_IS_ON {
                ctx.resources.seed_led.set_high().unwrap();
            } else {
                ctx.resources.seed_led.set_low().unwrap();
            }
            *LED_IS_ON = !(*LED_IS_ON);
        }
    }
};
