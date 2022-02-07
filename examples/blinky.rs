//! examples/blinky.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true
)]
mod app {
    use log::info;
    // Includes a panic handler and optional logging facilities
    use libdaisy::logger;

    use stm32h7xx_hal::stm32;
    use stm32h7xx_hal::timer::Timer;

    use libdaisy::gpio;
    use libdaisy::prelude::*;
    use libdaisy::system;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        seed_led: gpio::SeedLed,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        info!("Startup done!");

        system.timer2.set_freq(500.ms());

        (
            Shared {},
            Local {
                seed_led: system.gpio.led,
                timer2: system.timer2,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM2, local = [timer2, seed_led, led_is_on: bool = true])]
    fn blink(ctx: blink::Context) {
        ctx.local.timer2.clear_irq();

        if *ctx.local.led_is_on {
            ctx.local.seed_led.set_high().unwrap();
        } else {
            ctx.local.seed_led.set_low().unwrap();
        }
        *ctx.local.led_is_on = !(*ctx.local.led_is_on);
    }
}
