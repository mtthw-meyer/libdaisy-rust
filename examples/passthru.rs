//! examples/passthru.rs
#![no_main]
#![no_std]

use libdaisy_rust::*;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: audio::Audio,
    }

    #[init( schedule = [foo] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let mut system = system::System::init(ctx.core, ctx.device);

        system.audio.set_callback(passthru);
        init::LateResources {
            audio: system.audio,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = SAI1, resources =  [audio] )]
    fn listener2(ctx: listener2::Context) {
        ctx.resources.audio.process();
    }

    #[task]
    fn foo(_: foo::Context) {}

    extern "C" {
        fn TIM4();
    }
};

fn passthru(stereo: (f32, f32)) -> (f32, f32) {
    // To breakout the channels you can use the following:
    // let (left, right) = stereo;
    // (left, right)
    stereo
}
