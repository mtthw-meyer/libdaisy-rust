//! examples/delay.rs
#![no_main]
#![no_std]
use log::info;

use libdaisy::audio;
use libdaisy::logger;
use libdaisy::system;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: audio::Audio,
        buffer: audio::AudioBuffer,
        sdram: &'static mut [f32],
        #[init(0)]
        index: usize,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let system = system::System::init(ctx.core, ctx.device);
        let buffer = [(0.0, 0.0); audio::BLOCK_SIZE_MAX];

        info!("Startup done!");

        init::LateResources {
            audio: system.audio,
            buffer,
            sdram: system.sdram,
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
    #[task( binds = DMA1_STR1, resources = [audio, buffer, sdram, index], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        let buffer = ctx.resources.buffer;
        let sdram: &mut [f32] = ctx.resources.sdram;
        let index: &mut usize = ctx.resources.index;

        if audio.get_stereo(buffer) {
            for (left, right) in buffer {
                audio
                    .push_stereo((sdram[*index], sdram[*index + 1]))
                    .unwrap();
                sdram[*index] = *left;
                sdram[*index + 1] = *right;
                *index = (*index + 2) % libdaisy::AUDIO_SAMPLE_RATE;
            }
        }
    }
};
