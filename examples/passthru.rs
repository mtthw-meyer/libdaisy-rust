//! examples/passthru.rs
#![no_main]
#![no_std]

use cortex_m::asm::nop;
use stm32h7xx_hal::sai::*;
use stm32h7xx_hal::stm32;

use libdaisy_rust::*;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: Sai<stm32::SAI1, I2S>,
    }

    #[init( schedule = [foo] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let mut system = system::System::init(ctx.core, ctx.device);

        system.audio.listen(SaiChannel::ChannelB, Event::Data);
        system.audio.enable();
        system.audio.try_send(0, 0).unwrap();

        init::LateResources {
            audio: system.audio,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            nop();
        }
    }

    #[task( binds = SAI1, resources =  [audio] )]
    fn listener2(ctx: listener2::Context) {
        if let Ok((left, right)) = ctx.resources.audio.try_read() {
            if let Err(_) = ctx.resources.audio.try_send(left, right) {
                // Failed to send
            }
        }
    }

    #[task]
    fn foo(_: foo::Context) {}

    extern "C" {
        fn TIM4();
    }
};
