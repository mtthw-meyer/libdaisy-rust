//! examples/volume.rs
#![no_main]
#![no_std]
use log::info;

use libdaisy_rust::audio;
use libdaisy_rust::logger;
use libdaisy_rust::system;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: audio::Audio,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let system = system::System::init(ctx.core, ctx.device);

        info!("Startup done!");

        init::LateResources {
            audio: system.audio,
        }
    }

    // Non-default idle ensures chip doesn't go to sleep which causes issues for
    // probe.rs currently
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    // Interrupt handler for audio, should not generally need to be modified
    #[task( binds = SAI1, resources = [audio], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        audio.read();

        if let Some(stereo_iter) = audio.input.get_stereo_iter() {
            for (left, right) in stereo_iter {
                audio.output.push((left, right)).unwrap();
            }
        }

        audio.send();
    }
};
