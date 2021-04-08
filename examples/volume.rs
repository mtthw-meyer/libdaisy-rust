//! examples/volume.rs
#![allow(unused_imports)]
#![no_main]
#![no_std]
use rtic::cyccnt::U32Ext;

use log::info;

use stm32h7xx_hal::adc;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::audio;
use libdaisy::gpio::*;
use libdaisy::hid;
use libdaisy::logger;
use libdaisy::prelude::*;
use libdaisy::system;
use libdaisy::MILICYCLES;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: audio::Audio,
        buffer: audio::AudioBuffer,
        adc1: adc::Adc<stm32::ADC1, adc::Enabled>,
        control1: hid::AnalogControl<Daisy15<Analog>>,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        let buffer = [(0.0, 0.0); audio::BLOCK_SIZE_MAX];

        info!("Enable adc1");
        let mut adc1 = system.adc1.enable();
        adc1.set_resolution(adc::Resolution::SIXTEENBIT);
        let adc1_max = adc1.max_sample() as f32;

        let daisy15 = system
            .gpio
            .daisy15
            .take()
            .expect("Failed to get pin daisy15!")
            .into_analog();

        let mut control1 = hid::AnalogControl::new(daisy15, adc1_max);
        // Transform linear input into logarithmic
        control1.set_transform(|x| x * x);

        init::LateResources {
            audio: system.audio,
            buffer,
            adc1,
            control1,
            timer2: system.timer2,
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
    #[task( binds = DMA1_STR1, resources = [audio, buffer, control1], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        let buffer = ctx.resources.buffer;

        if audio.get_stereo(buffer) {
            for (left, right) in buffer {
                let volume = ctx.resources.control1.get_value();
                info!("{}", volume);
                *left *= volume;
                *right *= volume;
                audio.push_stereo((*left, *right)).unwrap();
            }
        }
    }

    #[task( binds = TIM2, resources = [timer2, adc1, control1] )]
    fn interface_handler(mut ctx: interface_handler::Context) {
        ctx.resources.timer2.clear_irq();
        let adc1 = ctx.resources.adc1;

        // Lower priority task(s) need to lock the resource.
        ctx.resources.control1.lock(|control1| {
            if let Ok(data) = adc1.read(control1.get_pin()) {
                control1.update(data);
            };
        });
    }
};
