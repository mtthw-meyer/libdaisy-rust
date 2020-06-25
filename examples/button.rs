#![no_std]
#![no_main]

use libdaisy_rust::*;
use libdaisy_rust::gpio::{Input, PullUp};
use stm32h7xx_hal::stm32::TIM2;
use stm32h7xx_hal::timer::Timer;

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        timer2: Timer<TIM2>,
        seed_led: gpio::SeedLed,
        button1: gpio::Daisy28<Input<PullUp>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);

        let button1 = system.gpio.daisy28.into_pull_up_input();

        init::LateResources {
            timer2: system.timer2,
            seed_led: system.gpio.led,
            button1,
         }
    }

    #[task( binds = TIM2, priority = 1, resources = [timer2, seed_led, button1] )]
    fn main(ctx: main::Context) {
        ctx.resources.timer2.clear_irq();
        if ctx.resources.button1.is_low().unwrap() {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
    }
};
