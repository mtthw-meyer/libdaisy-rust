//! examples/blinky2.rs
#![no_main]
#![no_std]

use rtic::cyccnt::U32Ext;

use core::panic::PanicInfo;
use rprintln as println;
use rtt_target::{rprintln, rtt_init_print};

// use embedded_hal::i2s::FullDuplex;

use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::sai::*;
use stm32h7xx_hal::stm32;

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
        // log: Log,
        audio: Sai<stm32::SAI1, I2S>,
    }

    #[init( schedule = [blink] )]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();

        let mut system = system::System::init(ctx.core, ctx.device);

        let now = ctx.start;
        // ctx.schedule
        //     .blink(now + (CLK_CYCLES_PER_MS * 250).cycles())
        //     .unwrap();

        system.audio.enable();
        system.audio.try_send(0, 0).unwrap();

        // println!(
        //     system.log,
        //     "SAI1 Enabled: {} Pending: {}",
        //     stm32::NVIC::is_enabled(interrupt::SAI1),
        //     stm32::NVIC::is_pending(interrupt::SAI1),
        // );

        init::LateResources {
            seed_led: system.gpio.led,
            // log: system.log,
            audio: system.audio,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
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
            .blink(ctx.scheduled + (CLK_CYCLES_PER_MS * 1000).cycles())
            .unwrap();
    }

    #[task( binds = SAI1, resources =  [audio] )]
    fn listener2(ctx: listener2::Context) {
        if let Ok((left, right)) = ctx.resources.audio.try_read() {
            if let Err(_) = ctx.resources.audio.try_send(left, right) {
                println!("Failed to send");
            }
        }
    }

    extern "C" {
        fn TIM4();
    }
};
