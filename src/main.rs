#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use rtic::app;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

use libdaisy_rust::*;
use libdaisy_rust::gpio::{Input, PullUp};
use stm32h7xx_hal::gpio::{Edge, ExtiPin};

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        timer2: Timer<TIM2>,
        seed_led: gpio::SeedLed,
        button1: gpio::Daisy28<Input<PullUp>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let mut system = system::System::init(ctx.core, ctx.device);

        let mut button1 = system.gpio.daisy28.into_pull_up_input();
        button1.make_interrupt_source(&mut system.SYSCFG);
        button1.trigger_on_edge(&mut system.EXTI, Edge::RISING);
        button1.enable_interrupt(&mut system.EXTI);

        init::LateResources {
            timer2: system.timer2,
            seed_led: system.gpio.led,
            button1,
         }
    }

    #[task( binds = TIM2, priority = 1, resources = [timer2, seed_led] )]
    fn main(ctx: main::Context) {
        static mut LED_IS_ON: bool = false;
        ctx.resources.timer2.clear_irq();
        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);
    }

    // List of interrupts bindable
    // https://docs.rs/stm32h7xx-hal/0.5.0/stm32h7xx_hal/stm32/enum.Interrupt.html
    // Why EXTI15_10? No idea.
    // #[task( binds = EXTI15_10, resources = [button1, seed_led] )]
    // fn button1_press(ctx: button1_press::Context) {
    //     static mut LED_IS_ON: bool = false;
    //     ctx.resources.button1.clear_interrupt_pending_bit();
    //     if *LED_IS_ON {
    //         ctx.resources.seed_led.set_high().unwrap();
    //     } else {
    //         ctx.resources.seed_led.set_low().unwrap();
    //     }
    //     *LED_IS_ON = !(*LED_IS_ON);
    // }
};
