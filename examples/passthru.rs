//! examples/passthru.rs
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
        buffer: audio::AudioBuffer,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let system = system::System::init(ctx.core, ctx.device);
        let buffer = [(0.0, 0.0); system::BLOCK_SIZE_MAX];

        info!("Startup done!");

        init::LateResources {
            audio: system.audio,
            buffer,
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

    // Interrupt handler for audio
    #[task( binds = DMA1_STR1, resources = [audio, buffer], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        let buffer = ctx.resources.buffer;

        // audio.passthru();
        audio.get_stereo(buffer);
        for (left, right) in buffer {
            audio.push_stereo((*left, *right)).unwrap();
        }
    }
};
