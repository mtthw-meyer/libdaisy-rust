#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use libdaisy_rust::*;

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        system: System,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = System::init(ctx.core, ctx.device);

        init::LateResources { system  }
    }

    #[task( binds = TIM2, priority = 1, resources = [system] )]
    fn main(ctx: main::Context) {
        static mut LED_IS_ON: bool = false;
        ctx.resources.system.timer2.clear_irq();
        if *LED_IS_ON {
            ctx.resources.system.led_pin.set_high().unwrap();
        } else {
            ctx.resources.system.led_pin.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);
    }
};
