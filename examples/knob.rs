//! examples/knob.rs
#![no_main]
#![no_std]
// Includes a panic handler and optional logging facilities
use libdaisy_rust::logger;

use stm32h7xx_hal::adc;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy_rust::gpio::*;
use libdaisy_rust::hid;
use libdaisy_rust::prelude::*;
use libdaisy_rust::system;
use stm32h7xx_hal::time::Hertz;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        led1: hid::Led<Daisy28<Output<PushPull>>>,
        adc1: adc::Adc<stm32::ADC1, adc::Enabled>,
        control1: hid::AnalogControl<Daisy21<Analog>>,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);

        let duty_cycle = 50;
        let resolution = 20;

        system.timer2.set_freq(Hertz(duty_cycle * resolution));

        let daisy28 = system
            .gpio
            .daisy28
            .take()
            .expect("Failed to get pin 28!")
            .into_push_pull_output();

        let led1 = hid::Led::new(daisy28, false, resolution);

        let mut adc1 = system.adc1.enable();
        adc1.set_resolution(adc::Resolution::SIXTEENBIT);
        let adc1_max = adc1.max_sample() as f32;

        let daisy21 = system
            .gpio
            .daisy21
            .take()
            .expect("Failed to get pin 21!")
            .into_analog();

        let control1 = hid::AnalogControl::new(daisy21, adc1_max);

        init::LateResources {
            led1,
            adc1,
            control1,
            timer2: system.timer2,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, adc1, control1, led1] )]
    fn interface_handler(ctx: interface_handler::Context) {
        ctx.resources.timer2.clear_irq();
        let adc1 = ctx.resources.adc1;
        let led1 = ctx.resources.led1;
        let control1 = ctx.resources.control1;

        if let Ok(data) = adc1.read(control1.get_pin()) {
            control1.update(data);
        }

        led1.set_brightness(control1.get_value());
        led1.update();
    }
};
