//! examples/blinky2.rs
#![no_main]
#![no_std]

use cortex_m::asm::nop;
use rtic::cyccnt::U32Ext;

use panic_halt as _;

use libdaisy_rust::*;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
    }

    #[init( schedule = [blink] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);

        let now = ctx.start;
        ctx.schedule
            .blink(now + (MILICYCLES * 250).cycles())
            .unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            nop();
        }
    }

    #[task( schedule = [blink], resources = [seed_led] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *LED_IS_ON = !(*LED_IS_ON);

        ctx.schedule
            .blink(ctx.scheduled + (MILICYCLES * 250).cycles())
            .unwrap();
    }

    extern "C" {
        fn TIM4();
    }
};
